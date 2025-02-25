use std::sync::Arc;
use crate::{Package, ResourceDescriptor, ResourceIdentifier};

#[derive(Default)]
pub struct PackageSet {
    packages: Vec<Arc<Package>>,
}

impl PackageSet {
    pub fn new(packages: Vec<Arc<Package>>) -> PackageSet {
        Self { packages }
    }

    pub fn get_packages(&self) -> &Vec<Arc<Package>> {
        &self.packages
    }
    
    pub fn add_package(&mut self, package: Arc<Package>) {
        self.packages.push(package);
    }

    pub fn find_resource(&self, uid: &ResourceIdentifier) -> Result<ResourceDescriptor, String> {
        for package in &self.packages {
            if package.meta.namespace != uid.namespace {
                continue;
            }

            if let Ok(desc) = package.find_resource(uid) {
                return Ok(desc);
            }
        }

        Err("Resource not found in package set".to_owned())
    }
}

impl Into<Vec<Arc<Package>>> for PackageSet {
    fn into(self) -> Vec<Arc<Package>> {
        self.packages
    }
}
