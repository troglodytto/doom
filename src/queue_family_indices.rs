use crate::{AppData, SuitabilityError};
use anyhow::{anyhow, Result};
use vulkanalia::{
    prelude::v1_0::{vk, Instance, InstanceV1_0},
    vk::{KhrSurfaceExtension, QueueFamilyProperties},
};

#[derive(Copy, Clone, Debug)]
pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
}

impl QueueFamilyIndices {
    pub unsafe fn get(
        instance: &Instance,
        data: &mut AppData,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self> {
        let properties = instance.get_physical_device_queue_family_properties(physical_device); // @audit

        let present = QueueFamilyIndices::get_physical_device_surface_support_khr_index(
            &properties,
            instance,
            data,
            physical_device,
        )?; // @audit

        let graphics = QueueFamilyIndices::get_graphics_queue_family_index(&properties);

        match (graphics, present) {
            (Some(graphics), Some(present)) => Ok(Self { graphics, present }),
            _ => Err(anyhow!(SuitabilityError("Missing queue families."))),
        }
    }

    fn get_graphics_queue_family_index(properties: &Vec<QueueFamilyProperties>) -> Option<u32> {
        properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32)
    }

    unsafe fn get_physical_device_surface_support_khr_index(
        properties: &Vec<QueueFamilyProperties>,
        instance: &Instance,
        data: &mut AppData,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Option<u32>> {
        // @todo - Test this method `for index in 0..properties.len() {`
        for (index, _) in properties.iter().enumerate() {
            // @audit
            if instance.get_physical_device_surface_support_khr(
                physical_device,
                index as u32,
                data.surface,
            )? {
                return Ok(Some(index as u32));
            }
        }

        Ok(None)
    }
}
