//! 目录清理操作接口

use std::fs;

use crate::core::cfg::structs::{
    ClearInput, ClearOutput, DeletedItem, ItemType,
};

/// 递归删除目录下所有文件和子目录
/// 目录不存在视为成功，但 cleared = false
pub fn clear_lianpkg(input: ClearInput) -> ClearOutput {
    let dir_path = input.dir_path;
    
    // 目录不存在，不触发删除
    if !dir_path.exists() {
        return ClearOutput {
            cleared: false,
            deleted_items: Vec::new(),
        };
    }
    
    // 收集所有要删除的项
    let mut deleted_items = Vec::new();
    
    // 递归收集并删除
    if let Err(_) = collect_and_delete(&dir_path, &mut deleted_items) {
        return ClearOutput {
            cleared: false,
            deleted_items,
        };
    }
    
    // 删除根目录本身
    if fs::remove_dir(&dir_path).is_ok() {
        deleted_items.push(DeletedItem {
            path: dir_path,
            item_type: ItemType::Directory,
        });
    }
    
    ClearOutput {
        cleared: true,
        deleted_items,
    }
}

/// 递归收集并删除目录内容
/// 先删除文件，再删除子目录（深度优先）
fn collect_and_delete(
    dir: &std::path::Path,
    deleted_items: &mut Vec<DeletedItem>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read dir {}: {}", dir.display(), e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            // 递归处理子目录
            collect_and_delete(&path, deleted_items)?;
            
            // 删除空目录
            if fs::remove_dir(&path).is_ok() {
                deleted_items.push(DeletedItem {
                    path,
                    item_type: ItemType::Directory,
                });
            }
        } else {
            // 删除文件
            if fs::remove_file(&path).is_ok() {
                deleted_items.push(DeletedItem {
                    path,
                    item_type: ItemType::File,
                });
            }
        }
    }
    
    Ok(())
}
