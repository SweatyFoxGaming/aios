//! Service Registry for Phoenix OS.
//! This allows kernel modules to register and discover structured APIs.

use lazy_static::lazy_static;
use spin::Mutex;

/// Maximum number of services.
const MAX_SERVICES: usize = 32;

/// A service entry in the registry.
#[derive(Debug, Clone, Copy)]
pub struct ServiceEntry {
    /// The name of the service.
    pub name: &'static str,
    /// The version of the service API.
    pub version: u32,
}

lazy_static! {
    static ref REGISTRY: Mutex<ServiceRegistry> = Mutex::new(ServiceRegistry::new());
}

struct ServiceRegistry {
    services: [Option<ServiceEntry>; MAX_SERVICES],
    count: usize,
}

impl ServiceRegistry {
    const fn new() -> Self {
        Self {
            services: [None; MAX_SERVICES],
            count: 0,
        }
    }

    const fn register(&mut self, entry: ServiceEntry) -> bool {
        if self.count >= MAX_SERVICES {
            return false;
        }
        self.services[self.count] = Some(entry);
        self.count += 1;
        true
    }
}

/// Register a new service.
#[must_use]
pub fn register(name: &'static str, version: u32) -> bool {
    REGISTRY.lock().register(ServiceEntry { name, version })
}

/// List all registered services.
pub fn list_services() {
    let registry = REGISTRY.lock();
    crate::println!("--- Registered Services ---");
    for service in registry.services.iter().flatten() {
        crate::println!("Service: {} (v{})", service.name, service.version);
    }
    crate::println!("---------------------------");
}
