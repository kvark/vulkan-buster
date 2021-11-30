use ash::{extensions::khr, vk, Entry};
use std::ffi::CStr;

fn main() {
    let entry = unsafe { Entry::new().unwrap() };
    let app_info = vk::ApplicationInfo::builder().api_version(vk::make_api_version(0, 1, 1, 0));

    let instance_extensions = entry.enumerate_instance_extension_properties().unwrap();
    let required_instance_extensions = [vk::KhrGetPhysicalDeviceProperties2Fn::name()];

    for ext in required_instance_extensions {
        if !instance_extensions
            .iter()
            .any(|inst_ext| unsafe { CStr::from_ptr(inst_ext.extension_name.as_ptr()) == ext })
        {
            println!("Unable to find {:?}", ext);
            return;
        }
    }

    let instance_extensions_raw = required_instance_extensions
        .iter()
        .map(|&s| s.as_ptr())
        .collect::<Vec<_>>();

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&instance_extensions_raw);

    let instance = unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Instance creation error")
    };

    let get_physical_device_properties = khr::GetPhysicalDeviceProperties2::new(&entry, &instance);

    let adapters = unsafe { instance.enumerate_physical_devices().unwrap() };

    let adapter = adapters[0];

    {
        let mut features_vk12 = vk::PhysicalDeviceVulkan12Features::default();
        let mut features2 = vk::PhysicalDeviceFeatures2KHR::builder()
            .features(vk::PhysicalDeviceFeatures::default())
            .push_next(&mut features_vk12)
            .build();

        unsafe {
            get_physical_device_properties.get_physical_device_features2(adapter, &mut features2);
        }

        println!("Timeline semaphores: {}", features_vk12.timeline_semaphore);
        if features_vk12.timeline_semaphore == 0 {
            return;
        }
    }

    let features_core = vk::PhysicalDeviceFeatures::default();
    let mut features_vk12 = vk::PhysicalDeviceVulkan12Features::default();
    features_vk12.timeline_semaphore = 1;

    let device_extensions_raw = vec![];
    let family_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(0)
        .queue_priorities(&[1.0])
        .build();
    let family_infos = [family_info];

    let device_create_info = vk::DeviceCreateInfo::builder()
        .enabled_extension_names(&device_extensions_raw)
        .enabled_features(&features_core)
        .queue_create_infos(&family_infos)
        .push_next(&mut features_vk12)
        .build();

    let device = unsafe {
        instance
            .create_device(adapter, &device_create_info, None)
            .unwrap()
    };

    let mut semaphore_type_info =
        vk::SemaphoreTypeCreateInfo::builder().semaphore_type(vk::SemaphoreType::TIMELINE);
    let semaphore_info = vk::SemaphoreCreateInfo::builder().push_next(&mut semaphore_type_info);
    let semaphore = unsafe { device.create_semaphore(&semaphore_info, None).unwrap() };

    // Check timeline semaphores to be available - https://github.com/gfx-rs/wgpu/issues/2215
    unsafe { device.get_semaphore_counter_value(semaphore).unwrap() };

    unsafe {
        device.destroy_semaphore(semaphore, None);
        device.destroy_device(None);
        instance.destroy_instance(None);
    }
    println!("Done");
}
