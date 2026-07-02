//! Service Registry for Phoenix OS.
//! This allows kernel modules to register and discover structured APIs.

use alloc::string::ToString;
use alloc::vec::Vec;
use common::security::{Capability, Token};
use lazy_static::lazy_static;
use spin::Mutex;

/// A service entry in the registry.
#[derive(Debug, Clone, Copy)]
pub struct ServiceEntry {
    /// The name of the service.
    pub name: &'static str,
    /// The version of the service API.
    pub version: u32,
    /// The required capability to use this service.
    pub required_capability: Option<Capability>,
    /// Whether this is a decoy "Honey-Service".
    pub is_decoy: bool,
}

lazy_static! {
    static ref REGISTRY: Mutex<ServiceRegistry> = Mutex::new(ServiceRegistry::new());
}

struct ServiceRegistry {
    services: Vec<ServiceEntry>,
}

impl ServiceRegistry {
    const fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    fn register(&mut self, entry: &ServiceEntry, token: &Token) -> bool {
        // Only allow registration if the caller has ServiceRegister capability
        if !token.has(Capability::ServiceRegister) {
            crate::audit::log(
                token,
                "Register Service: ".to_string() + entry.name,
                "Denied (Unauthorized)",
            );
            return false;
        }

        self.services.push(*entry);
        crate::audit::log(token, "Register Service: ".to_string() + entry.name, "Success");
        true
    }

    fn find(&self, name: &'static str, token: &Token) -> Option<ServiceEntry> {
        for service in &self.services {
            if service.name == name {
                // Sentinel check
                crate::sentinel::analyze_intent(token, name, "FindService");

                if service.is_decoy {
                    // Access to honey-service
                    return None;
                }

                // Verify caller has required capability for this service
                if let Some(cap) = service.required_capability {
                    if !token.has(cap) {
                        crate::audit::log(
                            token,
                            "Access Service: ".to_string() + name,
                            "Denied (Missing Capability)",
                        );
                        return None;
                    }
                }
                crate::audit::log(token, "Access Service: ".to_string() + name, "Granted");
                return Some(*service);
            }
        }
        None
    }
}

/// Register a new service.
#[must_use]
pub fn register(name: &'static str, version: u32, token: &Token) -> bool {
    REGISTRY.lock().register(
        &ServiceEntry {
            name,
            version,
            required_capability: None,
            is_decoy: false,
        },
        token,
    )
}

/// Register a service that requires a specific capability.
#[must_use]
pub fn register_secure(name: &'static str, version: u32, cap: Capability, token: &Token) -> bool {
    REGISTRY.lock().register(
        &ServiceEntry {
            name,
            version,
            required_capability: Some(cap),
            is_decoy: false,
        },
        token,
    )
}

/// Register a decoy "Honey-Service" for security scanning detection.
#[must_use]
pub fn register_decoy(name: &'static str, token: &Token) -> bool {
    REGISTRY.lock().register(
        &ServiceEntry {
            name,
            version: 0,
            required_capability: None,
            is_decoy: true,
        },
        token,
    )
}

/// Find a service by name.
#[must_use]
pub fn find(name: &'static str, token: &Token) -> Option<ServiceEntry> {
    REGISTRY.lock().find(name, token)
}

/// List all registered services.
pub fn list_services() {
    let registry = REGISTRY.lock();
    crate::println!("--- Registered Services ---");
    for service in &registry.services {
        let decoy_flag = if service.is_decoy { "[DECOY]" } else { "" };
        crate::println!(
            "Service: {} (v{}) [ReqCap: {:?}] {}",
            service.name,
            service.version,
            service.required_capability,
            decoy_flag
        );
    }
    crate::println!("---------------------------");
}
