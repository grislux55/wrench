use std::{collections::HashMap, sync::mpsc};

use crate::message::{ConnectInfo, RequiredAction, ResponseAction, TaskInfo};

use super::{wrench::WrenchStatus, ComProcess};

fn check_connect(
    com: &mut ComProcess,
    mut target: ConnectInfo,
    tx: &mpsc::Sender<ResponseAction>,
) -> anyhow::Result<()> {
    target.status = false;

    if let Some(index) = com.wrenches_serial_map.get(&target.wrench_serial) {
        if let Some(wrench) = com.wrenches.get(*index) {
            if !matches!(wrench.status, WrenchStatus::Disconnected) {
                target.status = true;
                tx.send(ResponseAction::ConnectStatus(target))?;
                return Ok(());
            }
        }
    }

    tx.send(ResponseAction::ConnectStatus(target))?;
    Ok(())
}

pub fn process_message_from_redis(
    com: &mut ComProcess,
    action: RequiredAction,
    tx: &mpsc::Sender<ResponseAction>,
) -> anyhow::Result<()> {
    match action {
        RequiredAction::BindWrench(target) => {
            com.connection_pending.push(target);
        }
        RequiredAction::CheckConnect(target) => check_connect(com, target, tx)?,
        RequiredAction::SendTask((msg_id, target)) => {
            let mut need_update = HashMap::new();
            for t in target {
                let serial = u128::from_str_radix(&t.wrench_serial, 16).unwrap_or(0);
                if let Some(index) = com.wrenches_serial_map.get(&serial) {
                    if com.wrenches.get_mut(*index).is_some() {
                        need_update.entry(serial).or_insert(vec![]).push(t);
                        continue;
                    }
                }

                let task_info = TaskInfo {
                    msg_id,
                    wrench_serial: serial,
                    status: false,
                };

                tx.send(ResponseAction::TaskStatus(task_info))?;
                return Ok(());
            }

            for (serial, tasks) in need_update {
                if let Some(index) = com.wrenches_serial_map.get(&serial) {
                    if let Some(wrench) = com.wrenches.get_mut(*index) {
                        wrench.redis_update(
                            RequiredAction::SendTask((msg_id.clone(), tasks)),
                            &com.writer,
                            tx,
                        );
                    }
                }
            }
        }
        RequiredAction::TaskCancel((wrench_serial, task_id)) => {
            let serial = u128::from_str_radix(&wrench_serial, 16).unwrap_or(0);
            if let Some(index) = com.wrenches_serial_map.get(&serial) {
                if let Some(wrench) = com.wrenches.get_mut(*index) {
                    wrench.redis_update(
                        RequiredAction::TaskCancel((wrench_serial, task_id)),
                        &com.writer,
                        tx,
                    );
                }
            }
        }
    }

    Ok(())
}
