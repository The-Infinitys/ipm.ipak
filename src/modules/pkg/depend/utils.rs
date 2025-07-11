use crate::utils::shell;
use crate::modules::pkg::RelationData;

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