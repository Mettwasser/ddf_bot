use thiserror::Error;

#[derive(Debug, Error)]
#[error("Missing role: {0}")]
pub struct MissingRole(pub u64);

#[macro_export]
macro_rules! has_role {
    ($fn_name:ident, $role_id:literal) => {
        pub async fn $fn_name(ctx: $crate::ContextEnum<'_>) -> Result<bool, $crate::Error> {
            let $crate::ContextEnum::Application(ctx) = ctx else {
                unreachable!()
            };
            let member = ctx.author_member().await.unwrap();
            if member
                .roles
                .contains(&poise::serenity_prelude::RoleId::new($role_id))
            {
                Ok(true)
            } else {
                Err(Box::new($crate::utils::MissingRole($role_id)) as Error)
            }
        }
    };
}
