#[cfg(test)]
mod tests {
    use crate::modules::pkg::depend::{
        DependencyGraph, InstallError, RemoveError,
    };
    use crate::modules::pkg::list::{
        InstalledPackageData, PackageListData,
    };
    use crate::modules::pkg::{
        AboutData, PackageAboutData, PackageData, PackageRange,
        PackageVersion, RelationData,
    };
    use crate::utils::version::{Version, VersionRange};
    use chrono::Local;
    use std::str::FromStr;

    // テスト用のPackageListDataを生成するヘルパー関数
    fn setup_package_list(packages: Vec<PackageData>) -> PackageListData {
        PackageListData {
            installed_packages: packages
                .into_iter()
                .map(|info| InstalledPackageData {
                    info,
                    last_modified: Local::now(),
                })
                .collect(),
            last_modified: Local::now(), // 修正: Vec<_> から DateTime<Local> に変更
        }
    }

    // テスト用のPackageDataを簡潔に生成するヘルパー関数
    fn create_package(
        name: &str,
        version: &str,
        depends: Option<Vec<Vec<PackageRange>>>,
        conflicts: Option<Vec<PackageRange>>,
        virtuals: Option<Vec<PackageVersion>>,
        depend_cmds: Option<Vec<String>>,
    ) -> PackageData {
        PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: name.to_string(),
                    version: Version::from_str(version).unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                depend: depends.unwrap_or_default(),
                conflicts: conflicts.unwrap_or_default(),
                virtuals: virtuals.unwrap_or_default(),
                depend_cmds: depend_cmds.unwrap_or_default(),
                suggests: Vec::new(),
                recommends: Vec::new(),
                provide_cmds: Vec::new(),
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_from_installed_packages() {
        // テストの目的: DependencyGraphがインストール済みパッケージから正しく構築されるか
        let pkg_a = create_package(
            "pkgA",
            "1.0",
            None,
            None,
            Some(vec![PackageVersion {
                name: "virtA".to_string(),
                version: Version::from_str("1.0").unwrap(),
            }]),
            None,
        );
        let pkg_b = create_package("pkgB", "2.0", None, None, None, None);
        let installed_packages = setup_package_list(vec![pkg_a, pkg_b]);
        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        // 実パッケージの確認（ゲッターメソッドを使用）
        assert!(graph.get_real_packages().contains_key("pkgA"));
        assert!(
            graph
                .get_real_packages()
                .get("pkgA")
                .unwrap()
                .contains(&Version::from_str("1.0").unwrap())
        );
        assert!(graph.get_real_packages().contains_key("pkgB"));
        assert!(
            graph
                .get_real_packages()
                .get("pkgB")
                .unwrap()
                .contains(&Version::from_str("2.0").unwrap())
        );

        // 仮想パッケージを含む利用可能なパッケージの確認（ゲッターメソッドを使用）
        assert!(graph.get_available_packages().contains_key("pkgA"));
        assert!(graph.get_available_packages().contains_key("pkgB"));
        assert!(graph.get_available_packages().contains_key("virtA"));
        assert!(
            graph
                .get_available_packages()
                .get("virtA")
                .unwrap()
                .contains(&Version::from_str("1.0").unwrap())
        );
    }

    #[test]
    fn test_are_dependencies_satisfied() {
        // テストの目的: パッケージの依存関係が満たされているかを正しく判定できるか
        let dep1 = create_package("dep1", "1.2", None, None, None, None);
        let installed_packages = setup_package_list(vec![dep1]);
        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        // 依存関係が満たされている場合
        let pkg = create_package(
            "pkg",
            "1.0",
            Some(vec![vec![PackageRange {
                name: "dep1".to_string(),
                range: VersionRange::from_str(">= 1.0").unwrap(),
            }]]),
            None,
            None,
            None,
        );
        assert!(graph.are_dependencies_satisfied(&pkg));

        // 依存関係が満たされていない場合
        let pkg_no_dep = create_package(
            "pkg",
            "1.0",
            Some(vec![vec![PackageRange {
                name: "dep2".to_string(),
                range: VersionRange::from_str(">= 1.0").unwrap(),
            }]]),
            None,
            None,
            None,
        );
        assert!(!graph.are_dependencies_satisfied(&pkg_no_dep));

        // 仮想パッケージの依存関係
        let provider = create_package(
            "provider",
            "2.0",
            None,
            None,
            Some(vec![PackageVersion {
                name: "virtual-pkg".to_string(),
                version: Version::from_str("1.5").unwrap(),
            }]),
            None,
        );
        let installed_packages2 = setup_package_list(vec![provider]);
        let graph2 =
            DependencyGraph::from_installed_packages(&installed_packages2);
        let pkg_virtual = create_package(
            "pkg",
            "1.0",
            Some(vec![vec![PackageRange {
                name: "virtual-pkg".to_string(),
                range: VersionRange::from_str(">= 1.0").unwrap(),
            }]]),
            None,
            None,
            None,
        );
        assert!(graph2.are_dependencies_satisfied(&pkg_virtual));
    }

    #[test]
    fn test_get_missing_dependencies() {
        // テストの目的: 欠けている依存関係を正しく取得できるか
        let pkg = create_package(
            "pkg",
            "1.0",
            Some(vec![
                vec![PackageRange {
                    name: "dep1".to_string(),
                    range: VersionRange::from_str(">=1.0").unwrap(),
                }],
                vec![PackageRange {
                    name: "dep2".to_string(),
                    range: VersionRange::from_str(">=2.0").unwrap(),
                }],
            ]),
            None,
            None,
            None,
        );
        let graph = DependencyGraph::from_installed_packages(
            &PackageListData::default(),
        );
        let missing = graph.get_missing_dependencies(&pkg);
        assert_eq!(missing.len(), 2);
        assert_eq!(missing[0][0].name, "dep1");
        assert_eq!(missing[1][0].name, "dep2");
    }

    #[test]
    fn test_has_conflicts() {
        // テストの目的: パッケージの競合を正しく検出できるか
        let conflict_pkg =
            create_package("conflict1", "1.2", None, None, None, None);
        let installed_packages = setup_package_list(vec![conflict_pkg]);
        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        let pkg = create_package(
            "pkg",
            "1.0",
            None,
            Some(vec![PackageRange {
                name: "conflict1".to_string(),
                range: VersionRange::from_str(">= 1.0").unwrap(),
            }]),
            None,
            None,
        );
        assert!(graph.has_conflicts(&pkg).is_some());

        let pkg_no_conflict = create_package(
            "pkg",
            "1.0",
            None,
            Some(vec![PackageRange {
                name: "conflict2".to_string(),
                range: VersionRange::from_str(">= 1.0").unwrap(),
            }]),
            None,
            None,
        );
        assert!(graph.has_conflicts(&pkg_no_conflict).is_none());
    }

    #[test]
    fn test_is_packages_installable() {
        // テストの目的: パッケージがインストール可能かどうかを正しく判定できるか
        let graph = DependencyGraph::from_installed_packages(
            &PackageListData::default(),
        );

        // 空リストのインストール
        assert!(graph.is_packages_installable(vec![]).is_ok());

        // 単一パッケージのインストール
        let pkg1 = create_package("pkg1", "1.0", None, None, None, None);
        assert!(graph.is_packages_installable(vec![pkg1.clone()]).is_ok());

        // 依存関係を持つパッケージのインストール
        let pkg2 = create_package(
            "pkg2",
            "1.0",
            Some(vec![vec![PackageRange {
                name: "pkg1".to_string(),
                range: VersionRange::from_str(">= 1.0").unwrap(),
            }]]),
            None,
            None,
            None,
        );
        assert!(
            graph
                .is_packages_installable(vec![pkg1.clone(), pkg2])
                .is_ok()
        );

        // 競合するパッケージ
        let pkg3 = create_package(
            "pkg3",
            "1.0",
            None,
            Some(vec![PackageRange {
                name: "pkg4".to_string(),
                range: VersionRange::from_str(">= 1.0").unwrap(),
            }]),
            None,
            None,
        );
        let pkg4 = create_package(
            "pkg4",
            "1.0",
            None,
            Some(vec![PackageRange {
                name: "pkg3".to_string(),
                range: VersionRange::from_str(">= 1.0").unwrap(),
            }]),
            None,
            None,
        );
        let result = graph
            .is_packages_installable(vec![pkg3.clone(), pkg4.clone()]);
        assert!(matches!(
            result,
            Err(InstallError::ConflictsWithOtherPackages { .. })
        ));

        // システムコマンド依存
        let pkg5 = create_package(
            "pkg5",
            "1.0",
            None,
            None,
            None,
            Some(vec!["nonexistent_cmd".to_string()]),
        );
        let result = graph.is_packages_installable(vec![pkg5]);
        assert!(matches!(
            result,
            Err(InstallError::MissingSystemCommands { .. })
        ));

        // 欠けている依存
        let pkg6 = create_package(
            "pkg6",
            "1.0",
            Some(vec![vec![PackageRange {
                name: "missing_dep".to_string(),
                range: VersionRange::from_str(">= 1.0").unwrap(),
            }]]),
            None,
            None,
            None,
        );
        let result = graph.is_packages_installable(vec![pkg6]);
        assert!(matches!(
            result,
            Err(InstallError::MissingDependencies { .. })
        ));

        // インストール済みパッケージとの競合
        let conflict_pkg =
            create_package("conflict1", "1.2", None, None, None, None);
        let installed_packages = setup_package_list(vec![conflict_pkg]);
        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);
        let pkg7 = create_package(
            "pkg7",
            "1.0",
            None,
            Some(vec![PackageRange {
                name: "conflict1".to_string(),
                range: VersionRange::from_str(">= 1.0").unwrap(),
            }]),
            None,
            None,
        );
        let result = graph.is_packages_installable(vec![pkg7]);
        assert!(matches!(
            result,
            Err(InstallError::ConflictsWithInstalled { .. })
        ));
    }

    #[test]
    fn test_is_packages_removable_no_dependents() {
        // テストの目的: 依存関係がないパッケージが削除可能か
        let pkg_a = create_package("pkgA", "1.0", None, None, None, None);
        let pkg_b = create_package("pkgB", "1.0", None, None, None, None);
        let installed_packages = setup_package_list(vec![pkg_a, pkg_b]);
        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        assert!(graph.is_packages_removable(&["pkgA"]).is_ok());
    }

    #[test]
    fn test_is_packages_removable_with_dependent() {
        // テストの目的: 依存されているパッケージの削除が適切に失敗するか
        let pkg_a = create_package("pkgA", "1.0", None, None, None, None);
        let pkg_b = create_package("pkgB", "1.0", None, None, None, None);
        let pkg_c = create_package(
            "pkgC",
            "1.0",
            Some(vec![vec![PackageRange {
                name: "pkgA".to_string(),
                range: VersionRange::from_str("= 1.0").unwrap(),
            }]]),
            None,
            None,
            None,
        );
        let installed_packages =
            setup_package_list(vec![pkg_a, pkg_b, pkg_c]);
        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        let result = graph.is_packages_removable(&["pkgA"]);

        if let Err(RemoveError::DependencyOfOtherPackages {
            ref dependent_packages,
            ..
        }) = result
        {
            assert!(dependent_packages.contains(&"pkgC".to_string()));
        } else {
            panic!(
                "result was not `RemoveError::DependencyOfOtherPackages`"
            )
        }
    }

    #[test]
    fn test_is_packages_removable_multiple_packages_with_dependent() {
        // テストの目的: 複数パッケージの削除が依存関係により失敗するか
        let pkg_a = create_package("pkgA", "1.0", None, None, None, None);
        let pkg_b = create_package("pkgB", "1.0", None, None, None, None);
        let pkg_c = create_package(
            "pkgC",
            "1.0",
            Some(vec![vec![PackageRange {
                name: "pkgA".to_string(),
                range: VersionRange::from_str("= 1.0").unwrap(),
            }]]),
            None,
            None,
            None,
        );
        let pkg_d = create_package(
            "pkgD",
            "1.0",
            Some(vec![vec![PackageRange {
                name: "pkgB".to_string(),
                range: VersionRange::from_str("= 1.0").unwrap(),
            }]]),
            None,
            None,
            None,
        );
        let installed_packages =
            setup_package_list(vec![pkg_a, pkg_b, pkg_c, pkg_d]);
        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        let result = graph.is_packages_removable(&["pkgA", "pkgB"]);

        if let Err(RemoveError::DependencyOfOtherPackages {
            ref dependent_packages,
            ..
        }) = result
        {
            assert!(
                dependent_packages.contains(&"pkgC".to_string())
                    || dependent_packages.contains(&"pkgD".to_string())
            );
        } else {
            panic!(
                "result was not `RemoveError::DependencyOfOtherPackages`"
            )
        }
    }

    #[test]
    fn test_is_packages_removable_self_contained() {
        // テストの目的: 自己完結的なパッケージセットが削除可能か
        let pkg_a = create_package(
            "pkgA",
            "1.0",
            Some(vec![vec![PackageRange {
                name: "pkgB".to_string(),
                range: VersionRange::from_str("= 1.0").unwrap(),
            }]]),
            None,
            None,
            None,
        );
        let pkg_b = create_package("pkgB", "1.0", None, None, None, None);
        let installed_packages = setup_package_list(vec![pkg_a, pkg_b]);
        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        assert!(graph.is_packages_removable(&["pkgA", "pkgB"]).is_ok());
    }

    #[test]
    fn test_is_packages_removable_virtual_dependency() {
        // テストの目的: 仮想パッケージ依存の削除が適切に失敗するか
        let pkg_a = create_package(
            "pkgA",
            "1.0",
            None,
            None,
            Some(vec![PackageVersion {
                name: "VirtDep".to_string(),
                version: Version::from_str("1.0").unwrap(),
            }]),
            None,
        );
        let pkg_b = create_package(
            "pkgB",
            "1.0",
            Some(vec![vec![PackageRange {
                name: "VirtDep".to_string(),
                range: VersionRange::from_str("= 1.0").unwrap(),
            }]]),
            None,
            None,
            None,
        );
        let installed_packages = setup_package_list(vec![pkg_a, pkg_b]);
        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        let result = graph.is_packages_removable(&["pkgA"]);
        if let Err(RemoveError::DependencyOfOtherPackages {
            ref dependent_packages,
            ..
        }) = result
        {
            assert!(dependent_packages.contains(&"pkgB".to_string()));
        } else {
            panic!(
                "result was not `RemoveError::DependencyOfOtherPackages`"
            )
        }
    }
}
