use maud::{Markup, html};
use serde::de::Visitor;
use serde::{Deserialize, Serialize};

pub mod user_role_check;
pub mod visitor_only;

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Root,
    User,
    Visitor,
}

impl Default for Role {
    fn default() -> Self {
        Self::User
    }
}

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(String::from(self).as_str())
    }
}

impl<'de> Deserialize<'de> for Role {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RoleVisitor;

        impl<'de> Visitor<'de> for RoleVisitor {
            type Value = Role;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a role")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Role::try_from(v).map_err(|_| E::custom("invalid role"))
            }
        }

        deserializer.deserialize_str(RoleVisitor)
    }
}

impl TryFrom<&str> for Role {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "root" => Ok(Self::Root),
            "user" => Ok(Self::User),
            _ => Err(()),
        }
    }
}

impl From<&Role> for String {
    fn from(r: &Role) -> Self {
        match r {
            Role::Root => "root".to_string(),
            Role::User => "user".to_string(),
            Role::Visitor => "visitor".to_string(),
        }
    }
}

impl Role {
    pub fn level(&self) -> u8 {
        match self {
            Self::Root => 2,
            Self::User => 1,
            Self::Visitor => 0,
        }
    }

    pub fn all_roles() -> Vec<Self> {
        vec![Self::Root, Self::User]
    }

    pub fn as_stringed(&self) -> String {
        String::from(self)
    }

    pub fn html_option(&self) -> Markup {
        html! {
            @for role in Self::all_roles() {
                @if self == &role {
                    option value=(role.as_stringed()) selected {
                        (role.as_stringed())
                    }
                } @else {
                    option value=(role.as_stringed()) {
                        (role.as_stringed())
                    }
                }
            }
        }
    }
}

impl PartialOrd for Role {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.level().partial_cmp(&other.level())
    }
}
