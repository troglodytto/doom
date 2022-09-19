use crate::{
    create_logical_device, create_pipeline, create_render_pass, create_swapchain,
    create_swapchain_image_views, AppData, QueueFamilyIndices, SuitabilityError, SwapchainSupport,
    DEVICE_EXTENSIONS, VALIDATION_ENABLED, VALIDATION_LAYER,
};
use anyhow::{anyhow, Result};
use log::{debug, error, info, trace, warn};
use std::{collections::HashSet, ffi::CStr, os::raw::c_void};
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY},
    prelude::v1_0::{vk, Device, DeviceV1_0, Entry, EntryV1_0, HasBuilder, Instance, InstanceV1_0},
    vk::{ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension},
    window as vk_window,
};
use winit::window::Window;

#[derive(Clone, Debug)]
pub struct App {
    entry: Entry,
    instance: Instance,
    data: AppData,
    device: Device,
}

impl App {
    pub unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?; // @audit
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?; // @audit
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?; // @audit
        data.surface = vk_window::create_surface(&instance, window)?; // @audit
        pick_physical_device(&instance, &mut data)?;
        let device = create_logical_device(&instance, &mut data)?; // @audit
        create_swapchain(window, &instance, &device, &mut data)?; // @audit
        create_swapchain_image_views(&device, &mut data)?;
        create_render_pass(&instance, &device, &mut data)?;
        create_pipeline(&device, &mut data)?;

        Ok(Self {
            entry,
            instance,
            data,
            device,
        })
    }

    pub fn render(&mut self, window: &Window) -> Result<()> {
        Ok(())
    }

    pub unsafe fn destroy(&mut self) {
        self.device.destroy_pipeline(self.data.pipeline, None); // @audit
        self.device
            .destroy_pipeline_layout(self.data.pipeline_layout, None); // @audit
        self.device.destroy_render_pass(self.data.render_pass, None); //@audit
        self.data
            .swapchain_image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None)); // @audit
        self.device.destroy_swapchain_khr(self.data.swapchain, None); // @audit
        self.device.destroy_device(None); // @audit

        if VALIDATION_ENABLED {
            self.instance
                .destroy_debug_utils_messenger_ext(self.data.messenger, None); // @audit
        }

        self.instance.destroy_surface_khr(self.data.surface, None); // @audit
        self.instance.destroy_instance(None); // @audit
    }
}

unsafe fn create_instance(window: &Window, entry: &Entry, data: &mut AppData) -> Result<Instance> {
    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Vulkan Tutorial (Rust)\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"No Engine\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    let available_layers = entry
        .enumerate_instance_layer_properties()? // @audit
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();

    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        vec![]
    };

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    if VALIDATION_ENABLED {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions);

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
        .user_callback(Some(debug_callback));

    if VALIDATION_ENABLED {
        info = info.push_next(&mut debug_info);
    }

    let instance = entry.create_instance(&info, None)?; // @audit

    if VALIDATION_ENABLED {
        // @audit
        data.messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok(instance)
}

unsafe fn pick_physical_device(instance: &Instance, data: &mut AppData) -> Result<()> {
    // @audit
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device); // @audit

        // @audit
        match check_physical_device(instance, data, physical_device) {
            Err(error) => {
                warn!(
                    "Skipping Physical device (`{}`): {}",
                    properties.device_name, error
                );
            }
            Ok(_) => {
                info!("Selecting Physical device (`{}`)", properties.device_name);
                data.physical_device = physical_device;
                return Ok(());
            }
        }
    }
    Err(anyhow!("Failed to find a suitable physical device"))
}

unsafe fn check_physical_device(
    instance: &Instance,
    data: &mut AppData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    // ? Used for checking when only using the discrete GPU
    // * let properties = instance.get_physical_device_properties(physical_device); // @audit
    // * if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
    // *     return Err(anyhow!(SuitabilityError(
    // *         "Only discrete GPUs are supported"
    // *     )));
    // * };

    // * let features = instance.get_physical_device_features(physical_device);
    // * if features.geometry_shader != vk::TRUE {
    // *     return Err(anyhow!(SuitabilityError("Missing geometry shader support")));
    // * }
    // * Ok(())
    QueueFamilyIndices::get(instance, data, physical_device)?; // @audit
    check_physical_device_extensions(instance, physical_device)?; // @audit

    let support = SwapchainSupport::get(instance, data, physical_device)?; // @audit
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err(anyhow!(SuitabilityError("Insufficient Swapchain Support.")));
    }

    Ok(())
}

unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)? // @audit
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();

    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(anyhow!(SuitabilityError("Missing extensions.")))
    }
}

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = unsafe { *data }; // @audit
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy(); // @audit

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    } else {
        trace!("({:?}) {}", type_, message);
    }

    vk::FALSE
}
