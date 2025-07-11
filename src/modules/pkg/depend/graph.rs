use chrono::Local;
use std::collections::{HashMap, HashSet, VecDeque};

use super::error::{InstallError, RemoveError}; // 同じモジュール内のエラーをインポート
use super::utils;
use crate::modules::pkg::list::{InstalledPackageData, PackageListData};
use crate::modules::pkg::{PackageData, PackageRange};
use crate::utils::version::Version; // utils::get_missing_depend_cmds を使用

#[derive(Clone)]
pub struct DependencyGraph {
    pub available_packages: HashMap<String, HashSet<Version>>,
    pub real_packages: HashMap<String, HashSet<Version>>,
    pub installed_package_data: Vec<InstalledPackageData>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// 空のDependencyGraphを作成します。
    ///
    /// # Returns
    ///
    /// 新しい空の `DependencyGraph` インスタンス。
    pub fn new() -> Self {
        Self {
            available_packages: HashMap::new(),
            real_packages: HashMap::new(),
            installed_package_data: Vec::new(),
        }
    }

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

    // ゲッターメソッドを追加
    pub fn get_real_packages(&self) -> &HashMap<String, HashSet<Version>> {
        &self.real_packages
    }

    pub fn get_available_packages(
        &self,
    ) -> &HashMap<String, HashSet<Version>> {
        &self.available_packages
    }

    pub fn without_packages(&self, packages_to_remove: &[&str]) -> Self {
        let mut new_graph = DependencyGraph::new();

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
            }) || other.relation.conflicts.iter().any(|conflict| {
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
        // `with_additional_packages` はトレイトメソッドとして呼び出す
        let temp_graph =
            self.with_additional_packages(&installing_packages);

        for package in &installing_packages {
            let missing_cmds =
                utils::get_missing_depend_cmds(&package.relation);
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

/// DependencyGraph の拡張操作を定義するトレイト
pub trait DependencyGraphOperations {
    /// 指定されたパッケージを追加した新しいDependencyGraphを返します。
    fn with_additional_packages(
        &self,
        additional_packages: &[PackageData],
    ) -> Self;

    /// インストール対象のパッケージを依存関係に基づいてトポロジカルソートします。
    /// 既にインストールされているパッケージを考慮し、未解決の依存関係がある場合はエラーを返します。
    ///
    /// # Arguments
    /// * `packages_to_sort` - ソート対象のパッケージデータのリスト。
    ///
    /// # Returns
    /// ソートされたパッケージデータのリスト、または解決できない依存関係がある場合のエラー。
    fn topological_sort_packages_for_install(
        &self,
        packages_to_sort: &[PackageData],
    ) -> Result<Vec<PackageData>, InstallError>;
}

// DependencyGraphOperations トレイトを DependencyGraph に実装
impl DependencyGraphOperations for DependencyGraph {
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
                last_modified: Local::now(),
            });
        }

        new_graph
    }

    fn topological_sort_packages_for_install(
        &self,
        packages_to_sort: &[PackageData],
    ) -> Result<Vec<PackageData>, InstallError> {
        let mut sorted_list = Vec::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

        let mut package_map: HashMap<String, PackageData> = HashMap::new();
        for pkg in packages_to_sort {
            package_map
                .insert(pkg.about.package.name.clone(), pkg.clone());
        }

        for pkg in packages_to_sort {
            let pkg_name = &pkg.about.package.name;
            in_degree.entry(pkg_name.clone()).or_insert(0);

            for dep_group in &pkg.relation.depend {
                let mut group_satisfied_by_installed = false;
                for dep in dep_group {
                    if self.is_dependency_satisfied(dep) {
                        group_satisfied_by_installed = true;
                        break;
                    }
                }

                if !group_satisfied_by_installed {
                    let mut depends_on_internal = false;
                    for dep in dep_group {
                        if package_map.contains_key(&dep.name) {
                            adj_list
                                .entry(dep.name.clone())
                                .or_default()
                                .push(pkg_name.clone());
                            depends_on_internal = true;
                        }
                    }
                    if depends_on_internal {
                        in_degree
                            .entry(pkg_name.clone())
                            .and_modify(|e| *e += 1);
                    }
                }
            }
        }

        let mut queue: VecDeque<String> = VecDeque::new();
        for (pkg_name, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(pkg_name.clone());
            }
        }

        while let Some(pkg_name) = queue.pop_front() {
            if let Some(pkg_data) = package_map.get(&pkg_name) {
                sorted_list.push(pkg_data.clone());
            }

            if let Some(dependents) = adj_list.get(&pkg_name) {
                for dependent_pkg_name in dependents {
                    *in_degree.get_mut(dependent_pkg_name).unwrap() -= 1;
                    if *in_degree.get(dependent_pkg_name).unwrap() == 0 {
                        queue.push_back(dependent_pkg_name.clone());
                    }
                }
            }
        }

        if sorted_list.len() != packages_to_sort.len() {
            let missing_packages: Vec<String> = packages_to_sort
                .iter()
                .filter(|pkg| {
                    !sorted_list.iter().any(|s_pkg| {
                        s_pkg.about.package.name == pkg.about.package.name
                    })
                })
                .map(|pkg| pkg.about.package.name.clone())
                .collect();

            return Err(InstallError::CyclicDependencies {
                packages: missing_packages,
            });
        }

        Ok(sorted_list)
    }
}
