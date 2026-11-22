#[cfg(feature = "model")]
use crate::builder::EditRole;
#[cfg(all(feature = "cache", feature = "model"))]
use crate::cache::Cache;
#[cfg(feature = "model")]
use crate::http::Http;
use crate::model::prelude::*;
#[allow(async_fn_in_trait)]
pub trait RoleExt: Sized {
    #[cfg(feature = "model")]
    /// Deletes the role.
    ///
    /// **Note** Requires the [Manage Roles] permission.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to delete this role.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    async fn delete(&mut self, http: impl AsRef<Http>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Edits a [`Role`], optionally setting its new fields.
    ///
    /// Requires the [Manage Roles] permission.
    ///
    /// # Examples
    ///
    /// See the documentation of [`EditRole`] for details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user does not have permission to Manage Roles.
    ///
    /// [Manage Roles]: Permissions::MANAGE_ROLES
    async fn edit(&mut self, http: impl AsRef<Http>, builder: EditRole<'_>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Check that the role has the given permission.
    #[must_use]
    fn has_permission(&self, permission: Permissions) -> bool;
    #[cfg(feature = "model")]
    /// Checks whether the role has all of the given permissions.
    ///
    /// The 'precise' argument is used to check if the role's permissions are precisely equivalent
    /// to the given permissions. If you need only check that the role has at least the given
    /// permissions, pass `false`.
    #[must_use]
    fn has_permissions(&self, permissions: Permissions, precise: bool) -> bool;
    #[cfg(feature = "model")]
    #[must_use]
    /// Generates a URL to the Role icon's image.
    fn icon_url(&self) -> Option<String>;
}
impl RoleExt for Role {
    #[cfg(feature = "model")]
    #[inline]
    async fn delete(&mut self, http: impl AsRef<Http>) -> Result<()> {
        http.as_ref().delete_role(self.guild_id, self.id, None).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn edit(&mut self, http: impl AsRef<Http>, builder: EditRole<'_>) -> Result<()> {
        *self = self.guild_id.edit_role(http.as_ref(), self.id, builder).await?;
        Ok(())
    }
    #[cfg(feature = "model")]
    #[inline]
    fn has_permission(&self, permission: Permissions) -> bool {
        self.permissions.contains(permission)
    }
    #[cfg(feature = "model")]
    #[inline]
    fn has_permissions(&self, permissions: Permissions, precise: bool) -> bool {
        if precise {
            self.permissions == permissions
        } else {
            self.permissions.contains(permissions)
        }
    }
    #[cfg(feature = "model")]
    #[inline]
    fn icon_url(&self) -> Option<String> {
        self.icon.map(|icon| {
            let ext = if icon.is_animated() { "gif" } else { "webp" };
            cdn!("/role-icons/{}/{}.{}", self.id, icon, ext)
        })
    }
}
#[allow(async_fn_in_trait)]
pub trait RoleIdExt: Sized {
    #[cfg(feature = "model")]
    /// Tries to find the [`Role`] by its Id in the cache.
    #[cfg(feature = "cache")]
    #[deprecated(
        since = "0.0.1",
        note = "Use Guild::roles. This performs a loop over the entire cache!"
    )]
    fn to_role_cached(self, cache: impl AsRef<Cache>) -> Option<Role>;
}
impl RoleIdExt for RoleId {
    #[cfg(feature = "model")]
    #[cfg(feature = "cache")]
    fn to_role_cached(self, cache: impl AsRef<Cache>) -> Option<Role> {
        for guild_entry in cache.as_ref().guilds.iter() {
            let guild = guild_entry.value();
            if !guild.roles.contains_key(&self) {
                continue;
            }
            if let Some(role) = guild.roles.get(&self) {
                return Some(role.clone());
            }
        }
        None
    }
}
