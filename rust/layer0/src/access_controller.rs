//! 访问控制模块
//!
//! RBAC 权限管理系统。

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 角色
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
}

/// 权限
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
        }
    }
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub roles: HashSet<String>,
}

/// 访问控制器
pub struct AccessController {
    /// 角色-权限映射
    roles: RwLock<HashMap<String, Role>>,
    /// 用户-角色映射
    user_roles: RwLock<HashMap<String, HashSet<String>>>,
}

impl AccessController {
    pub fn new() -> Self {
        Self {
            roles: RwLock::new(Self::default_roles()),
            user_roles: RwLock::new(HashMap::new()),
        }
    }

    /// 默认角色配置
    fn default_roles() -> HashMap<String, Role> {
        let mut roles = HashMap::new();

        // Admin 角色 - 所有权限
        let admin_perms: HashSet<Permission> = [Permission::new("*", "*")].into_iter().collect();
        roles.insert(
            "admin".to_string(),
            Role {
                name: "admin".to_string(),
                permissions: admin_perms,
            },
        );

        // User 角色 - 基本权限
        let user_perms: HashSet<Permission> = [
            Permission::new("session", "read"),
            Permission::new("session", "write"),
            Permission::new("tool", "execute"),
            Permission::new("agent", "run"),
        ]
        .into_iter()
        .collect();
        roles.insert(
            "user".to_string(),
            Role {
                name: "user".to_string(),
                permissions: user_perms,
            },
        );

        // Guest 角色 - 最小权限
        let guest_perms: HashSet<Permission> =
            [Permission::new("session", "read")].into_iter().collect();
        roles.insert(
            "guest".to_string(),
            Role {
                name: "guest".to_string(),
                permissions: guest_perms,
            },
        );

        roles
    }

    /// 检查权限
    pub fn check(&self, user_id: &str, resource: &str, action: &str) -> bool {
        let user_roles = self.user_roles.read();

        // 获取用户角色
        let roles = user_roles.get(user_id).cloned().unwrap_or_else(|| {
            // 默认给 guest 角色
            HashSet::from(["guest".to_string()])
        });

        let role_map = self.roles.read();

        // 检查每个角色的权限
        for role_name in roles {
            if let Some(role) = role_map.get(&role_name) {
                for perm in &role.permissions {
                    // 检查通配符权限
                    if perm.resource == "*" || perm.resource == resource {
                        if perm.action == "*" || perm.action == action {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// 为用户添加角色
    pub fn add_role(&self, user_id: &str, role_name: &str) {
        let mut user_roles = self.user_roles.write();
        user_roles
            .entry(user_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(role_name.to_string());
    }

    /// 为用户移除角色
    pub fn remove_role(&self, user_id: &str, role_name: &str) {
        let mut user_roles = self.user_roles.write();
        if let Some(roles) = user_roles.get_mut(user_id) {
            roles.remove(role_name);
        }
    }

    /// 创建自定义角色
    pub fn create_role(&self, role: Role) {
        let mut roles = self.roles.write();
        roles.insert(role.name.clone(), role);
    }

    /// 获取用户所有权限
    pub fn get_permissions(&self, user_id: &str) -> HashSet<Permission> {
        let user_roles = self.user_roles.read();
        let roles = user_roles.get(user_id).cloned().unwrap_or_default();
        let role_map = self.roles.read();

        let mut permissions = HashSet::new();
        for role_name in roles {
            if let Some(role) = role_map.get(&role_name) {
                permissions.extend(role.permissions.clone());
            }
        }
        permissions
    }
}

impl Default for AccessController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_roles() {
        let controller = AccessController::new();

        // Admin 应该有所有权限
        controller.add_role("admin_user", "admin");
        assert!(controller.check("admin_user", "any_resource", "any_action"));

        // User 应该有基本权限
        controller.add_role("normal_user", "user");
        assert!(controller.check("normal_user", "tool", "execute"));
        assert!(!controller.check("normal_user", "admin", "create"));
    }

    #[test]
    fn test_guest_default() {
        let controller = AccessController::new();

        // 未设置角色的用户默认是 guest
        assert!(controller.check("unknown_user", "session", "read"));
        assert!(!controller.check("unknown_user", "session", "write"));
    }
}
