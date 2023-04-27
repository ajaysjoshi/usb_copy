use rusb::*;
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt::Write;
use std::os::raw::c_char;
use std::ptr::null;
use std::str;

#[no_mangle]
pub extern "C" fn multiply_int(a: i64, b: i64) -> i64 {
    return a * b;
}

#[no_mangle]
pub extern "C" fn multiply_float(a: f64, b: f64) -> f64 {
    return a * b;
}

#[no_mangle]
pub extern "C" fn print_string(c_string_ptr: *const c_char) -> *mut c_char {
    let bytes = unsafe { CStr::from_ptr(c_string_ptr).to_bytes() };
    let str_slice = str::from_utf8(bytes).unwrap();
    println!("my my {}", str_slice);
    let c_string =
        CString::new(b"This is what I got in return".to_vec()).expect("CString::new failed");
    return c_string.into_raw();
}

use std::time::Duration;

use usb_ids::{self, FromId};

struct UsbDevice<T: UsbContext> {
    handle: DeviceHandle<T>,
    language: Language,
    timeout: Duration,
}

#[no_mangle]
pub extern "C" fn rusb_list() -> *mut c_char {
    let mut str = String::new();
    list_devices(&mut str).unwrap();
    unsafe {
        let mut ret = CString::new(str.as_bytes().to_vec()).expect("CString::new failed");
        return ret.into_raw();
    };
}

fn list_devices(str: &mut String) -> Result<()> {
    let timeout = Duration::from_secs(1);

    for device in DeviceList::new()?.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        let mut usb_device = {
            match device.open() {
                Ok(h) => match h.read_languages(timeout) {
                    Ok(l) => {
                        if !l.is_empty() {
                            Some(UsbDevice {
                                handle: h,
                                language: l[0],
                                timeout,
                            })
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                },
                Err(_) => None,
            }
        };
        writeln!(
            str,
            "Bus {:03} Device {:03} ID {:04x}:{:04x} {}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id(),
            get_speed(device.speed())
        );
        print_device(str, &device_desc, &mut usb_device);
        for n in 0..device_desc.num_configurations() {
            let config_desc = match device.config_descriptor(n) {
                Ok(c) => c,
                Err(_) => continue,
            };

            print_config(str, &config_desc, &mut usb_device);

            for interface in config_desc.interfaces() {
                for interface_desc in interface.descriptors() {
                    print_interface(str, &interface_desc, &mut usb_device);

                    for endpoint_desc in interface_desc.endpoint_descriptors() {
                        print_endpoint(str, &endpoint_desc);
                    }
                }
            }
        }
    }

    Ok(())
}

fn print_device<T: UsbContext>(
    str: &mut String,
    device_desc: &DeviceDescriptor,
    handle: &mut Option<UsbDevice<T>>,
) {
    let vid = device_desc.vendor_id();
    let pid = device_desc.product_id();

    let vendor_name = match usb_ids::Vendor::from_id(device_desc.vendor_id()) {
        Some(vendor) => vendor.name(),
        None => "Unknown vendor",
    };

    let product_name =
        match usb_ids::Device::from_vid_pid(device_desc.vendor_id(), device_desc.product_id()) {
            Some(product) => product.name(),
            None => "Unknown product",
        };

    writeln!(str, "Device Descriptor:");
    writeln!(
        str,
        "  bcdUSB             {:2}.{}{}",
        device_desc.usb_version().major(),
        device_desc.usb_version().minor(),
        device_desc.usb_version().sub_minor()
    );
    writeln!(
        str,
        "  bDeviceClass        {:#04x}",
        device_desc.class_code()
    );
    writeln!(
        str,
        "  bDeviceSubClass     {:#04x}",
        device_desc.sub_class_code()
    );
    writeln!(
        str,
        "  bDeviceProtocol     {:#04x}",
        device_desc.protocol_code()
    );
    writeln!(
        str,
        "  bMaxPacketSize0      {:3}",
        device_desc.max_packet_size()
    );
    writeln!(str, "  idVendor          {vid:#06x} {vendor_name}",);
    writeln!(str, "  idProduct         {pid:#06x} {product_name}",);
    writeln!(
        str,
        "  bcdDevice          {:2}.{}{}",
        device_desc.device_version().major(),
        device_desc.device_version().minor(),
        device_desc.device_version().sub_minor()
    );
    writeln!(
        str,
        "  iManufacturer        {:3} {}",
        device_desc.manufacturer_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_manufacturer_string(h.language, device_desc, h.timeout)
            .unwrap_or_default())
    );
    writeln!(
        str,
        "  iProduct             {:3} {}",
        device_desc.product_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_product_string(h.language, device_desc, h.timeout)
            .unwrap_or_default())
    );
    writeln!(
        str,
        "  iSerialNumber        {:3} {}",
        device_desc.serial_number_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_serial_number_string(h.language, device_desc, h.timeout)
            .unwrap_or_default())
    );
    writeln!(
        str,
        "  bNumConfigurations   {:3}",
        device_desc.num_configurations()
    );
}

fn print_config<T: UsbContext>(
    str: &mut String,
    config_desc: &ConfigDescriptor,
    handle: &mut Option<UsbDevice<T>>,
) {
    writeln!(str, "  Config Descriptor:");
    writeln!(
        str,
        "    bNumInterfaces       {:3}",
        config_desc.num_interfaces()
    );
    writeln!(str, "    bConfigurationValue  {:3}", config_desc.number());
    writeln!(
        str,
        "    iConfiguration       {:3} {}",
        config_desc.description_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_configuration_string(h.language, config_desc, h.timeout)
            .unwrap_or_default())
    );
    writeln!(str, "    bmAttributes:");
    writeln!(
        str,
        "      Self Powered     {:>5}",
        config_desc.self_powered()
    );
    writeln!(
        str,
        "      Remote Wakeup    {:>5}",
        config_desc.remote_wakeup()
    );
    writeln!(
        str,
        "    bMaxPower           {:4}mW",
        config_desc.max_power()
    );

    if !config_desc.extra().is_empty() {
        writeln!(str, "    {:?}", config_desc.extra());
    } else {
        writeln!(str, "    no extra data");
    }
}

fn print_interface<T: UsbContext>(
    str: &mut String,
    interface_desc: &InterfaceDescriptor,
    handle: &mut Option<UsbDevice<T>>,
) {
    writeln!(str, "    Interface Descriptor:");
    writeln!(
        str,
        "      bInterfaceNumber     {:3}",
        interface_desc.interface_number()
    );
    writeln!(
        str,
        "      bAlternateSetting    {:3}",
        interface_desc.setting_number()
    );
    writeln!(
        str,
        "      bNumEndpoints        {:3}",
        interface_desc.num_endpoints()
    );
    writeln!(
        str,
        "      bInterfaceClass     {:#04x}",
        interface_desc.class_code()
    );
    writeln!(
        str,
        "      bInterfaceSubClass  {:#04x}",
        interface_desc.sub_class_code()
    );
    writeln!(
        str,
        "      bInterfaceProtocol  {:#04x}",
        interface_desc.protocol_code()
    );
    writeln!(
        str,
        "      iInterface           {:3} {}",
        interface_desc.description_string_index().unwrap_or(0),
        handle.as_mut().map_or(String::new(), |h| h
            .handle
            .read_interface_string(h.language, interface_desc, h.timeout)
            .unwrap_or_default())
    );

    if interface_desc.extra().is_empty() {
        writeln!(str, "    {:?}", interface_desc.extra());
    } else {
        writeln!(str, "    no extra data");
    }
}

fn print_endpoint(str: &mut String, endpoint_desc: &EndpointDescriptor) {
    writeln!(str, "      Endpoint Descriptor:");
    writeln!(
        str,
        "        bEndpointAddress    {:#04x} EP {} {:?}",
        endpoint_desc.address(),
        endpoint_desc.number(),
        endpoint_desc.direction()
    );
    writeln!(str, "        bmAttributes:");
    writeln!(
        str,
        "          Transfer Type          {:?}",
        endpoint_desc.transfer_type()
    );
    writeln!(
        str,
        "          Synch Type             {:?}",
        endpoint_desc.sync_type()
    );
    writeln!(
        str,
        "          Usage Type             {:?}",
        endpoint_desc.usage_type()
    );
    writeln!(
        str,
        "        wMaxPacketSize    {:#06x}",
        endpoint_desc.max_packet_size()
    );
    writeln!(
        str,
        "        bInterval            {:3}",
        endpoint_desc.interval()
    );
}

fn get_speed(speed: Speed) -> &'static str {
    match speed {
        Speed::SuperPlus => "10000 Mbps",
        Speed::Super => "5000 Mbps",
        Speed::High => " 480 Mbps",
        Speed::Full => "  12 Mbps",
        Speed::Low => " 1.5 Mbps",
        _ => "(unknown)",
    }
}
