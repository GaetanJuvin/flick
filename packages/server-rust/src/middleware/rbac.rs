use crate::auth::AuthUser;
use crate::error::AppError;

/// Verify the user has admin role. Returns an error if not.
pub fn require_admin(user: &AuthUser) -> Result<(), AppError> {
    if user.role != "admin" {
        return Err(AppError::forbidden("Admin access required"));
    }
    Ok(())
}
