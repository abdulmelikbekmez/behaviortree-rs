use bt_derive::{ControlNode, TreeNodeDefaults};

use crate::{
    basic_types::NodeStatus,
    nodes::{ControlNode, NodeConfig, TreeNode, TreeNodePtr, NodeError, NodeHalt},
};

/// The ReactiveFallback is similar to a ParallelNode.
/// All the children are ticked from first to last:
/// 
/// - If a child returns RUNNING, continue to the next sibling.
/// - If a child returns FAILURE, continue to the next sibling.
/// - If a child returns SUCCESS, stop and return SUCCESS.
/// 
/// If all the children fail, than this node returns FAILURE.
/// 
/// IMPORTANT: to work properly, this node should not have more than
///            a single asynchronous child.
#[derive(TreeNodeDefaults, ControlNode, Debug, Clone)]
pub struct ReactiveFallbackNode {
    config: NodeConfig,
    children: Vec<TreeNodePtr>,
    status: NodeStatus,
}

impl ReactiveFallbackNode {
    pub fn new(config: NodeConfig) -> ReactiveFallbackNode {
        Self {
            config,
            children: Vec::new(),
            status: NodeStatus::Idle,
        }
    }
}

impl TreeNode for ReactiveFallbackNode {
    fn tick(&mut self) -> Result<NodeStatus, NodeError> {
        let mut all_skipped = true;
        self.status = NodeStatus::Running;

        for index in 0..self.children.len() {
            let cur_child = &mut self.children[index];

            let child_status = cur_child.borrow_mut().execute_tick()?;

            all_skipped &= child_status == NodeStatus::Skipped;

            match &child_status {
                NodeStatus::Running => {
                    for i in 0..index {
                        self.halt_child(i)?;
                    }

                    return Ok(NodeStatus::Running);
                }
                NodeStatus::Failure => {}
                NodeStatus::Success => {
                    self.reset_children();
                    return Ok(NodeStatus::Success);
                }
                NodeStatus::Skipped => {
                    self.halt_child(index)?;
                }
                NodeStatus::Idle => {
                    return Err(NodeError::StatusError("Name here".to_string(), "Idle".to_string()));
                }
            };
        }

        self.reset_children();

        match all_skipped {
            true => Ok(NodeStatus::Skipped),
            false => Ok(NodeStatus::Failure),
        }
    }
}

impl NodeHalt for ReactiveFallbackNode {
    fn halt(&mut self) {
        self.reset_children();
    }
}