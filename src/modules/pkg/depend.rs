use super::super::version::Version;
use super::list::PackageListData;
use super::{PackageData, PackageRange, RelationData};
use crate::modules::pkg::list::InstalledPackageData;
use crate::utils::shell;
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug)]
pub enum InstallError {
    MissingDependencies {
        package: String,
        missing: Vec<Vec<PackageRange>>,
    },
    ConflictsWithInstalled {
        package: String,
        conflicts: Vec<PackageRange>,
    },
    ConflictsWithOtherPackages {
        package: String,
        conflicts_with: String,
    },
    MissingSystemCommands {
        package: String,
        missing_cmds: Vec<String>,
    },
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstallError::MissingDependencies { package, missing } => {
                write!(
                    f,
                    "Package {} has missing dependencies: {:?}",
                    package, missing
                )
            }
            InstallError::ConflictsWithInstalled {
                package,
                conflicts,
            } => {
                write!(
                    f,
                    "Package {} conflicts with installed packages: {:?}",
                    package, conflicts
                )
            }
            InstallError::ConflictsWithOtherPackages {
                package,
                conflicts_with,
            } => {
                write!(
                    f,
                    "Package {} conflicts with another package: {}",
                    package, conflicts_with
                )
            }
            InstallError::MissingSystemCommands {
                package,
                missing_cmds,
            } => {
                write!(
                    f,
                    "Package {} requires unavailable system commands: {:?}",
                    package, missing_cmds
                )
            }
        }
    }
}

impl std::error::Error for InstallError {}

#[derive(Debug)]
pub enum RemoveError {
    DependencyOfOtherPackages {
        package: String,

        dependent_packages: Vec<String>,
    },
}

impl fmt::Display for RemoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoveError::DependencyOfOtherPackages {
                package,
                dependent_packages,
            } => {
                write!(
                    f,
                    "Package '{}' cannot be removed because the following packages depend on it: {:?}",
                    package, dependent_packages
                )
            }
        }
    }
}

impl std::error::Error for RemoveError {}

#[derive(Clone)]
pub struct DependencyGraph {
    available_packages: HashMap<String, HashSet<Version>>,

    real_packages: HashMap<String, HashSet<Version>>,

    installed_package_data: Vec<InstalledPackageData>,
}

impl DependencyGraph {
    pub fn from_installed_packages(
        installed_packages: &PackageListData,
    ) -> Self {
        let mut real_packages = HashMap::new();
        let mut available_packages = HashMap::new();
        let installed_package_data =
            installed_packages.installed_packages.clone();

        for package in &installed_package_data {
            let name = package.info.about.package.name.clone();
            let version = package.info.about.package.version.clone();

            real_packages
                .entry(name.clone())
                .or_insert_with(HashSet::new)
                .insert(version.clone());

            available_packages
                .entry(name)
                .or_insert_with(HashSet::new)
                .insert(version.clone());

            for virtual_pkg in &package.info.relation.virtuals {
                let v_name = virtual_pkg.name.clone();
                let v_version = virtual_pkg.version.clone();
                available_packages
                    .entry(v_name)
                    .or_insert_with(HashSet::new)
                    .insert(v_version);
            }
        }

        DependencyGraph {
            available_packages,
            real_packages,
            installed_package_data,
        }
    }

    fn with_additional_packages(
        &self,
        additional_packages: &[PackageData],
    ) -> Self {
        let mut new_graph = self.clone();

        for package in additional_packages {
            let name = package.about.package.name.clone();
            let version = package.about.package.version.clone();

            new_graph
                .real_packages
                .entry(name.clone())
                .or_default()
                .insert(version.clone());

            new_graph
                .available_packages
                .entry(name)
                .or_default()
                .insert(version.clone());

            for virtual_pkg in &package.relation.virtuals {
                let v_name = virtual_pkg.name.clone();
                let v_version = virtual_pkg.version.clone();
                new_graph
                    .available_packages
                    .entry(v_name)
                    .or_default()
                    .insert(v_version);
            }

            new_graph.installed_package_data.push(InstalledPackageData {
                info: package.clone(),
                last_modified: chrono::Local::now(),
            });
        }

        new_graph
    }

    fn without_packages(&self, packages_to_remove: &[&str]) -> Self {
        let mut new_graph = DependencyGraph {
            available_packages: HashMap::new(),
            real_packages: HashMap::new(),
            installed_package_data: Vec::new(),
        };

        for package in &self.installed_package_data {
            let pkg_name = &package.info.about.package.name;
            if !packages_to_remove.contains(&pkg_name.as_str()) {
                let name = package.info.about.package.name.clone();
                let version = package.info.about.package.version.clone();

                new_graph
                    .real_packages
                    .entry(name.clone())
                    .or_default()
                    .insert(version.clone());
                new_graph
                    .available_packages
                    .entry(name)
                    .or_default()
                    .insert(version.clone());

                for virtual_pkg in &package.info.relation.virtuals {
                    let v_name = virtual_pkg.name.clone();
                    let v_version = virtual_pkg.version.clone();
                    new_graph
                        .available_packages
                        .entry(v_name)
                        .or_default()
                        .insert(v_version);
                }
                new_graph.installed_package_data.push(package.clone());
            }
        }
        new_graph
    }

    pub fn is_dependency_satisfied(&self, dep: &PackageRange) -> bool {
        self.available_packages.get(&dep.name).is_some_and(|versions| {
            versions.iter().any(|v| dep.range.compare(v))
        })
    }

    pub fn are_dependencies_satisfied(
        &self,
        package: &PackageData,
    ) -> bool {
        package.relation.depend.iter().all(|group| {
            group.iter().any(|dep| self.is_dependency_satisfied(dep))
        })
    }

    pub fn get_missing_dependencies(
        &self,
        package: &PackageData,
    ) -> Vec<Vec<PackageRange>> {
        package
            .relation
            .depend
            .iter()
            .filter(|group| {
                !group.iter().any(|dep| self.is_dependency_satisfied(dep))
            })
            .cloned()
            .collect()
    }

    pub fn has_conflicts(
        &self,
        package: &PackageData,
    ) -> Option<Vec<PackageRange>> {
        let conflicts = package
            .relation
            .conflicts
            .iter()
            .filter(|conflict| {
                self.real_packages.get(&conflict.name).is_some_and(
                    |versions| {
                        versions.iter().any(|v| conflict.range.compare(v))
                    },
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        if conflicts.is_empty() { None } else { Some(conflicts) }
    }

    pub fn has_conflicts_with_packages(
        &self,
        package: &PackageData,
        other_packages: &[&PackageData],
    ) -> Option<String> {
        for other in other_packages {
            let other_name = &other.about.package.name;
            let other_version = &other.about.package.version;

            if package.relation.conflicts.iter().any(|conflict| {
                conflict.name == *other_name
                    && conflict.range.compare(other_version)
            }) {
                return Some(other_name.clone());
            }

            if other.relation.conflicts.iter().any(|conflict| {
                conflict.name == package.about.package.name
                    && conflict
                        .range
                        .compare(&package.about.package.version)
            }) {
                return Some(other_name.clone());
            }
        }
        None
    }

    pub fn is_packages_installable(
        &self,
        installing_packages: Vec<PackageData>,
    ) -> Result<(), InstallError> {
        let temp_graph =
            self.with_additional_packages(&installing_packages);

        for package in &installing_packages {
            let missing_cmds = get_missing_depend_cmds(&package.relation);
            if !missing_cmds.is_empty() {
                return Err(InstallError::MissingSystemCommands {
                    package: package.about.package.name.clone(),
                    missing_cmds,
                });
            }
        }

        for (i, package) in installing_packages.iter().enumerate() {
            let pkg_name = package.about.package.name.clone();

            let missing_deps =
                temp_graph.get_missing_dependencies(package);
            if !missing_deps.is_empty() {
                return Err(InstallError::MissingDependencies {
                    package: pkg_name,
                    missing: missing_deps,
                });
            }

            if let Some(conflicts) = self.has_conflicts(package) {
                return Err(InstallError::ConflictsWithInstalled {
                    package: pkg_name,
                    conflicts,
                });
            }

            let other_packages = installing_packages
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, pkg)| pkg)
                .collect::<Vec<_>>();
            if let Some(conflicts_with) = temp_graph
                .has_conflicts_with_packages(package, &other_packages)
            {
                return Err(InstallError::ConflictsWithOtherPackages {
                    package: pkg_name,
                    conflicts_with,
                });
            }
        }

        Ok(())
    }

    pub fn is_packages_removable(
        &self,
        packages_to_remove_names: &[&str],
    ) -> Result<(), RemoveError> {
        let temp_graph = self.without_packages(packages_to_remove_names);

        for installed_pkg_data in &self.installed_package_data {
            let current_pkg_name =
                &installed_pkg_data.info.about.package.name;

            if packages_to_remove_names
                .contains(&current_pkg_name.as_str())
            {
                continue;
            }

            if !temp_graph
                .are_dependencies_satisfied(&installed_pkg_data.info)
            {
                let dependent_packages = vec![current_pkg_name.clone()];

                return Err(RemoveError::DependencyOfOtherPackages {
                    package: packages_to_remove_names.join(", "),
                    dependent_packages,
                });
            }
        }

        Ok(())
    }
}

pub fn are_depend_cmds_available(relation: &RelationData) -> bool {
    relation.depend_cmds.iter().all(|cmd| shell::is_cmd_available(cmd))
}

pub fn get_missing_depend_cmds(relation: &RelationData) -> Vec<String> {
    relation
        .depend_cmds
        .iter()
        .filter(|cmd| !shell::is_cmd_available(cmd))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::list::{InstalledPackageData, PackageListData};
    use super::super::{
        AboutData, PackageAboutData, PackageData, PackageRange,
        PackageVersion, RelationData,
    };
    use super::*;
    use crate::modules::version::{Version, VersionRange};
    use chrono::Local;
    use std::str::FromStr;

    #[test]
    fn test_from_installed_packages() {
        let mut installed_packages = PackageListData::default();
        installed_packages.installed_packages = vec![
            InstalledPackageData {
                info: PackageData {
                    about: AboutData {
                        package: PackageAboutData {
                            name: "pkgA".to_string(),
                            version: Version::from_str("1.0").unwrap(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    relation: RelationData {
                        virtuals: vec![PackageVersion {
                            name: "virtA".to_string(),
                            version: Version::from_str("1.0").unwrap(),
                        }],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                last_modified: Local::now(),
            },
            InstalledPackageData {
                info: PackageData {
                    about: AboutData {
                        package: PackageAboutData {
                            name: "pkgB".to_string(),
                            version: Version::from_str("2.0").unwrap(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                last_modified: Local::now(),
            },
        ];

        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        assert!(graph.real_packages.contains_key("pkgA"));
        assert!(
            graph
                .real_packages
                .get("pkgA")
                .unwrap()
                .contains(&Version::from_str("1.0").unwrap())
        );
        assert!(graph.real_packages.contains_key("pkgB"));
        assert!(
            graph
                .real_packages
                .get("pkgB")
                .unwrap()
                .contains(&Version::from_str("2.0").unwrap())
        );

        assert!(graph.available_packages.contains_key("pkgA"));
        assert!(graph.available_packages.contains_key("pkgB"));
        assert!(graph.available_packages.contains_key("virtA"));
        assert!(
            graph
                .available_packages
                .get("virtA")
                .unwrap()
                .contains(&Version::from_str("1.0").unwrap())
        );
    }

    #[test]
    fn test_are_dependencies_satisfied() {
        let mut package = PackageData::default();
        package.relation.depend = vec![vec![PackageRange {
            name: "dep1".to_string(),
            range: VersionRange::from_str(">= 1.0").unwrap(),
        }]];

        let mut installed_packages = PackageListData::default();
        installed_packages.installed_packages =
            vec![InstalledPackageData {
                info: PackageData {
                    about: AboutData {
                        package: PackageAboutData {
                            name: "dep1".to_string(),
                            version: Version::from_str("1.2").unwrap(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                last_modified: Local::now(),
            }];

        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);
        assert!(graph.are_dependencies_satisfied(&package));

        let mut package2 = package.clone();
        package2.relation.depend[0][0].name = "dep2".to_string();
        assert!(!graph.are_dependencies_satisfied(&package2));

        let mut package3 = PackageData::default();
        package3.relation.depend = vec![vec![PackageRange {
            name: "virtual-pkg".to_string(),
            range: VersionRange::from_str(">= 1.0").unwrap(),
        }]];

        let mut installed_packages2 = PackageListData::default();
        installed_packages2.installed_packages =
            vec![InstalledPackageData {
                info: PackageData {
                    about: AboutData {
                        package: PackageAboutData {
                            name: "provider".to_string(),
                            version: Version::from_str("2.0").unwrap(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    relation: RelationData {
                        virtuals: vec![PackageVersion {
                            name: "virtual-pkg".to_string(),
                            version: Version::from_str("1.5").unwrap(),
                        }],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                last_modified: Local::now(),
            }];

        let graph2 =
            DependencyGraph::from_installed_packages(&installed_packages2);
        assert!(graph2.are_dependencies_satisfied(&package3));
    }

    #[test]
    fn test_get_missing_dependencies() {
        let mut package = PackageData::default();
        package.relation.depend = vec![
            vec![PackageRange {
                name: "dep1".to_string(),
                range: VersionRange::from_str(">=1.0").unwrap(),
            }],
            vec![PackageRange {
                name: "dep2".to_string(),
                range: VersionRange::from_str(">=2.0").unwrap(),
            }],
        ];

        let installed_packages = PackageListData::default();
        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);
        let missing = graph.get_missing_dependencies(&package);
        assert_eq!(missing.len(), 2);
        assert_eq!(missing[0][0].name, "dep1");
        assert_eq!(missing[1][0].name, "dep2");
    }

    #[test]
    fn test_has_conflicts() {
        let mut package = PackageData::default();
        package.relation.conflicts = vec![PackageRange {
            name: "conflict1".to_string(),
            range: VersionRange::from_str(">= 1.0").unwrap(),
        }];

        let mut installed_packages = PackageListData::default();
        installed_packages.installed_packages =
            vec![InstalledPackageData {
                info: PackageData {
                    about: AboutData {
                        package: PackageAboutData {
                            name: "conflict1".to_string(),
                            version: Version::from_str("1.2").unwrap(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                last_modified: Local::now(),
            }];

        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);
        assert!(graph.has_conflicts(&package).is_some());

        let mut package2 = package.clone();
        package2.relation.conflicts[0].name = "conflict2".to_string();
        assert!(graph.has_conflicts(&package2).is_none());
    }

    #[test]
    fn test_is_packages_installable() {
        let graph = DependencyGraph::from_installed_packages(
            &PackageListData::default(),
        );

        assert!(graph.is_packages_installable(vec![]).is_ok());

        let pkg1 = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkg1".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(graph.is_packages_installable(vec![pkg1.clone()]).is_ok());

        let pkg2 = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkg2".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                depend: vec![vec![PackageRange {
                    name: "pkg1".to_string(),
                    range: VersionRange::from_str(">= 1.0").unwrap(),
                }]],
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(
            graph
                .is_packages_installable(vec![pkg1.clone(), pkg2])
                .is_ok()
        );

        let pkg3 = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkg3".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                conflicts: vec![PackageRange {
                    name: "pkg4".to_string(),
                    range: VersionRange::from_str(">= 1.0").unwrap(),
                }],
                ..Default::default()
            },
            ..Default::default()
        };
        let pkg4 = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkg4".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                conflicts: vec![PackageRange {
                    name: "pkg3".to_string(),
                    range: VersionRange::from_str(">= 1.0").unwrap(),
                }],
                ..Default::default()
            },
            ..Default::default()
        };
        let result = graph
            .is_packages_installable(vec![pkg3.clone(), pkg4.clone()]);
        assert!(matches!(
            result,
            Err(InstallError::ConflictsWithOtherPackages { .. })
        ));

        let pkg5 = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkg5".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                depend_cmds: vec!["nonexistent_cmd".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let result = graph.is_packages_installable(vec![pkg5]);
        assert!(matches!(
            result,
            Err(InstallError::MissingSystemCommands { .. })
        ));

        let pkg6 = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkg6".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                depend: vec![vec![PackageRange {
                    name: "missing_dep".to_string(),
                    range: VersionRange::from_str(">= 1.0").unwrap(),
                }]],
                ..Default::default()
            },
            ..Default::default()
        };
        let result = graph.is_packages_installable(vec![pkg6]);
        assert!(matches!(
            result,
            Err(InstallError::MissingDependencies { .. })
        ));

        let mut installed_packages = PackageListData::default();
        installed_packages.installed_packages =
            vec![InstalledPackageData {
                info: PackageData {
                    about: AboutData {
                        package: PackageAboutData {
                            name: "conflict1".to_string(),
                            version: Version::from_str("1.2").unwrap(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                last_modified: Local::now(),
            }];
        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);
        let pkg7 = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkg7".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                conflicts: vec![PackageRange {
                    name: "conflict1".to_string(),
                    range: VersionRange::from_str(">= 1.0").unwrap(),
                }],
                ..Default::default()
            },
            ..Default::default()
        };
        let result = graph.is_packages_installable(vec![pkg7]);
        assert!(matches!(
            result,
            Err(InstallError::ConflictsWithInstalled { .. })
        ));
    }

    #[test]
    fn test_is_packages_removable_no_dependents() {
        let pkg_a_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgA".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let pkg_b_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgB".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let mut installed_packages = PackageListData::default();
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_a_data.clone(),
            last_modified: Local::now(),
        });
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_b_data.clone(),
            last_modified: Local::now(),
        });

        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        let result = graph.is_packages_removable(&["pkgA"]);
        assert!(
            result.is_ok(),
            "Should be able to remove pkgA: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_is_packages_removable_with_dependent() {
        let pkg_a_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgA".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let pkg_b_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgB".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let pkg_c_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgC".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                depend: vec![vec![PackageRange {
                    name: "pkgA".to_string(),
                    range: VersionRange::from_str("= 1.0").unwrap(),
                }]],
                ..Default::default()
            },
            ..Default::default()
        };

        let mut installed_packages = PackageListData::default();
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_a_data.clone(),
            last_modified: Local::now(),
        });
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_b_data.clone(),
            last_modified: Local::now(),
        });
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_c_data.clone(),
            last_modified: Local::now(),
        });

        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        let result = graph.is_packages_removable(&["pkgA"]);
        assert!(matches!(
            result,
            Err(RemoveError::DependencyOfOtherPackages {
                package: _,
                dependent_packages: _
            })
        ));
        if let Err(RemoveError::DependencyOfOtherPackages {
            package: _,
            dependent_packages,
        }) = result
        {
            assert!(dependent_packages.contains(&"pkgC".to_string()));
        }
    }

    #[test]
    fn test_is_packages_removable_multiple_packages_with_dependent() {
        let pkg_a_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgA".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let pkg_b_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgB".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let pkg_c_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgC".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                depend: vec![vec![PackageRange {
                    name: "pkgA".to_string(),
                    range: VersionRange::from_str("= 1.0").unwrap(),
                }]],
                ..Default::default()
            },
            ..Default::default()
        };
        let pkg_d_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgD".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                depend: vec![vec![PackageRange {
                    name: "pkgB".to_string(),
                    range: VersionRange::from_str("= 1.0").unwrap(),
                }]],
                ..Default::default()
            },
            ..Default::default()
        };

        let mut installed_packages = PackageListData::default();
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_a_data.clone(),
            last_modified: Local::now(),
        });
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_b_data.clone(),
            last_modified: Local::now(),
        });
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_c_data.clone(),
            last_modified: Local::now(),
        });
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_d_data.clone(),
            last_modified: Local::now(),
        });

        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        let result = graph.is_packages_removable(&["pkgA", "pkgB"]);
        assert!(matches!(
            result,
            Err(RemoveError::DependencyOfOtherPackages {
                package: _,
                dependent_packages: _
            })
        ));
        if let Err(RemoveError::DependencyOfOtherPackages {
            package: _,
            dependent_packages,
        }) = result
        {
            assert!(
                dependent_packages.contains(&"pkgC".to_string())
                    || dependent_packages.contains(&"pkgD".to_string())
            );
        }
    }

    #[test]
    fn test_is_packages_removable_self_contained() {
        let pkg_a_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgA".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                depend: vec![vec![PackageRange {
                    name: "pkgB".to_string(),
                    range: VersionRange::from_str("= 1.0").unwrap(),
                }]],
                ..Default::default()
            },
            ..Default::default()
        };
        let pkg_b_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgB".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let mut installed_packages = PackageListData::default();
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_a_data.clone(),
            last_modified: Local::now(),
        });
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_b_data.clone(),
            last_modified: Local::now(),
        });

        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        let result = graph.is_packages_removable(&["pkgA", "pkgB"]);
        assert!(
            result.is_ok(),
            "Should be able to remove pkgA and pkgB together: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_is_packages_removable_virtual_dependency() {
        let pkg_a_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgA".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                virtuals: vec![PackageVersion {
                    name: "VirtDep".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                }],
                ..Default::default()
            },
            ..Default::default()
        };
        let pkg_b_data = PackageData {
            about: AboutData {
                package: PackageAboutData {
                    name: "pkgB".to_string(),
                    version: Version::from_str("1.0").unwrap(),
                    ..Default::default()
                },
                ..Default::default()
            },
            relation: RelationData {
                depend: vec![vec![PackageRange {
                    name: "VirtDep".to_string(),
                    range: VersionRange::from_str("= 1.0").unwrap(),
                }]],
                ..Default::default()
            },
            ..Default::default()
        };

        let mut installed_packages = PackageListData::default();
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_a_data.clone(),
            last_modified: Local::now(),
        });
        installed_packages.installed_packages.push(InstalledPackageData {
            info: pkg_b_data.clone(),
            last_modified: Local::now(),
        });

        let graph =
            DependencyGraph::from_installed_packages(&installed_packages);

        let result = graph.is_packages_removable(&["pkgA"]);
        assert!(matches!(
            result,
            Err(RemoveError::DependencyOfOtherPackages {
                package: _,
                dependent_packages: _
            })
        ));
        if let Err(RemoveError::DependencyOfOtherPackages {
            package: _,
            dependent_packages,
        }) = result
        {
            assert!(dependent_packages.contains(&"pkgB".to_string()));
        }
    }
}
