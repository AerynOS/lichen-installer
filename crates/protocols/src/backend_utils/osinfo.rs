// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use os_info::OSInfo;

use crate::lichen::osinfo;

/// Converts an os_info::OSInfo reference into a protocol buffer lichen::osinfo::OsInfo
///
/// Maps all fields from the native OSInfo struct to the corresponding protocol buffer message
impl From<OSInfo> for osinfo::OsInfo {
    fn from(info: OSInfo) -> Self {
        let osinfo = osinfo::OsInfo {
            version: info.version.clone(),
            start_date: info.start_date.to_rfc3339(),
            metadata: Some(convert_metadata(&info.metadata)),
            system: Some(convert_system(&info.system)),
            resources: Some(convert_resources(&info.resources)),
            security_contact: info.security_contact.as_ref().map(convert_security_contact),
        };

        osinfo
    }
}

/// Converts OSInfo metadata to protocol buffer format
fn convert_metadata(metadata: &os_info::Metadata) -> osinfo::Metadata {
    osinfo::Metadata {
        identity: Some(convert_identity(&metadata.identity)),
        maintainers: convert_maintainers(&metadata.maintainers),
        version: Some(convert_version_info(&metadata.version)),
    }
}

/// Converts Identity information to protocol buffer format
fn convert_identity(identity: &os_info::Identity) -> osinfo::Identity {
    osinfo::Identity {
        id: identity.id.clone(),
        id_like: identity.id_like.clone(),
        name: identity.name.clone(),
        display: identity.display.clone(),
        ansi_color: identity.ansi_color.clone(),
        former_identities: identity.former_identities.iter().map(convert_former_identity).collect(),
    }
}

/// Converts FormerIdentity to protocol buffer format
fn convert_former_identity(identity: &os_info::FormerIdentity) -> osinfo::FormerIdentity {
    osinfo::FormerIdentity {
        id: identity.id.clone(),
        name: identity.name.clone(),
        start_date: identity.start_date.to_rfc3339(),
        end_date: identity.end_date.to_rfc3339(),
        end_version: identity.end_version.clone(),
        announcement: identity.announcement.clone(),
    }
}

/// Converts maintainer map to protocol buffer format
fn convert_maintainers(
    maintainers: &std::collections::HashMap<String, Vec<os_info::Maintainer>>,
) -> std::collections::HashMap<String, osinfo::MaintainerList> {
    maintainers
        .iter()
        .map(|(key, maintainers)| {
            let maintainer_list = osinfo::MaintainerList {
                maintainers: maintainers.iter().map(convert_maintainer).collect(),
            };
            (key.clone(), maintainer_list)
        })
        .collect()
}

/// Converts a single maintainer to protocol buffer format
fn convert_maintainer(maintainer: &os_info::Maintainer) -> osinfo::Maintainer {
    osinfo::Maintainer {
        name: maintainer.name.clone(),
        role: match maintainer.role {
            os_info::MaintainerRole::Founder => osinfo::MaintainerRole::Founder as i32,
            os_info::MaintainerRole::Maintainer => osinfo::MaintainerRole::Maintainer as i32,
            os_info::MaintainerRole::Contributor => osinfo::MaintainerRole::Contributor as i32,
        },
        email: maintainer.email.clone(),
        start_date: maintainer.start_date.as_ref().map(|d| d.to_rfc3339()),
        end_date: maintainer.end_date.as_ref().map(|d| d.to_rfc3339()),
    }
}

/// Converts version info to protocol buffer format
fn convert_version_info(version: &os_info::VersionInfo) -> osinfo::VersionInfo {
    osinfo::VersionInfo {
        full: version.full.clone(),
        short: version.short.clone(),
        build_id: version.build_id.clone(),
        released: version.released.to_rfc3339(),
        announcement: version.announcement.clone(),
        codename: version.codename.clone(),
    }
}

/// Converts system info to protocol buffer format
fn convert_system(system: &os_info::System) -> osinfo::System {
    osinfo::System {
        composition: Some(convert_composition(&system.composition)),
        features: Some(convert_features(&system.features)),
        kernel: Some(convert_kernel(&system.kernel)),
        platform: Some(convert_platform(&system.platform)),
        update: Some(convert_update(&system.update)),
    }
}

/// Converts system composition to protocol buffer format
fn convert_composition(composition: &os_info::Composition) -> osinfo::Composition {
    osinfo::Composition {
        bases: composition.bases.clone(),
        technology: Some(osinfo::Technology {
            core: composition.technology.core.clone(),
            optional: composition.technology.optional.clone(),
        }),
    }
}

/// Converts features to protocol buffer format
fn convert_features(features: &os_info::Features) -> osinfo::Features {
    osinfo::Features {
        atomic_updates: Some(osinfo::AtomicUpdates {
            strategy: features.atomic_updates.strategy.clone(),
            rollback_support: features.atomic_updates.rollback_support,
        }),
        boot: Some(osinfo::Boot {
            bootloader: features.boot.bootloader.clone(),
            firmware: Some(osinfo::Firmware {
                uefi: features.boot.firmware.uefi,
                secure_boot: features.boot.firmware.secure_boot,
                bios: features.boot.firmware.bios,
            }),
        }),
        filesystem: Some(osinfo::Filesystem {
            default: features.filesystem.default.clone(),
            supported: features.filesystem.supported.clone(),
        }),
    }
}

/// Converts kernel info to protocol buffer format
fn convert_kernel(kernel: &os_info::Kernel) -> osinfo::Kernel {
    osinfo::Kernel {
        r#type: kernel.kernel_type.clone(),
        name: kernel.name.clone(),
    }
}

/// Converts platform info to protocol buffer format
fn convert_platform(platform: &os_info::Platform) -> osinfo::Platform {
    osinfo::Platform {
        architecture: platform.architecture.clone(),
        variant: platform.variant.clone(),
    }
}

/// Converts update info to protocol buffer format
fn convert_update(update: &os_info::Update) -> osinfo::Update {
    osinfo::Update {
        strategy: update.strategy.clone(),
        cadence: Some(osinfo::Cadence {
            r#type: match update.cadence.cadence_type {
                os_info::CadenceType::Rolling => osinfo::CadenceType::Rolling as i32,
                os_info::CadenceType::Fixed => osinfo::CadenceType::Fixed as i32,
                os_info::CadenceType::Lts => osinfo::CadenceType::Lts as i32,
                os_info::CadenceType::Point => osinfo::CadenceType::Point as i32,
            },
            sync_interval: update.cadence.sync_interval.clone(),
            sync_day: update.cadence.sync_day.clone(),
            release_schedule: update.cadence.release_schedule.clone(),
            support_timeline: update.cadence.support_timeline.clone(),
        }),
        approach: update.approach.clone(),
    }
}

/// Converts resources to protocol buffer format
fn convert_resources(resources: &os_info::Resources) -> osinfo::Resources {
    osinfo::Resources {
        websites: convert_websites(&resources.websites),
        social: convert_social_links(&resources.social),
        funding: convert_funding_links(&resources.funding),
    }
}

/// Converts website map to protocol buffer format
fn convert_websites(
    websites: &std::collections::HashMap<String, os_info::Website>,
) -> std::collections::HashMap<String, osinfo::Website> {
    websites
        .iter()
        .map(|(key, website)| {
            let proto_website = osinfo::Website {
                url: website.url.clone(),
                display_name: website.display_name.clone(),
                scope: convert_website_scope(&website.scope),
            };
            (key.clone(), proto_website)
        })
        .collect()
}

/// Converts website scope enum to protocol buffer format
fn convert_website_scope(scope: &os_info::WebsiteScope) -> i32 {
    match scope {
        os_info::WebsiteScope::Home => osinfo::WebsiteScope::Home as i32,
        os_info::WebsiteScope::Documentation => osinfo::WebsiteScope::Documentation as i32,
        os_info::WebsiteScope::Support => osinfo::WebsiteScope::Support as i32,
        os_info::WebsiteScope::BugTracker => osinfo::WebsiteScope::BugTracker as i32,
        os_info::WebsiteScope::Developer => osinfo::WebsiteScope::Developer as i32,
        os_info::WebsiteScope::Public => osinfo::WebsiteScope::Public as i32,
        os_info::WebsiteScope::EndUserDocs => osinfo::WebsiteScope::EndUserDocs as i32,
        os_info::WebsiteScope::DeveloperDocs => osinfo::WebsiteScope::DeveloperDocs as i32,
        os_info::WebsiteScope::PrivacyPolicy => osinfo::WebsiteScope::PrivacyPolicy as i32,
        os_info::WebsiteScope::TermsOfService => osinfo::WebsiteScope::TermsOfService as i32,
        os_info::WebsiteScope::Legal => osinfo::WebsiteScope::Legal as i32,
        os_info::WebsiteScope::SecurityPolicy => osinfo::WebsiteScope::SecurityPolicy as i32,
    }
}

/// Converts social links map to protocol buffer format
fn convert_social_links(
    social: &std::collections::HashMap<String, os_info::SocialLink>,
) -> std::collections::HashMap<String, osinfo::SocialLink> {
    social
        .iter()
        .map(|(key, link)| {
            let proto_link = osinfo::SocialLink {
                url: link.url.clone(),
                display_name: link.display_name.clone(),
                platform: link.platform.clone(),
            };
            (key.clone(), proto_link)
        })
        .collect()
}

/// Converts funding links map to protocol buffer format
fn convert_funding_links(
    funding: &std::collections::HashMap<String, os_info::FundingLink>,
) -> std::collections::HashMap<String, osinfo::FundingLink> {
    funding
        .iter()
        .map(|(key, link)| {
            let proto_link = osinfo::FundingLink {
                url: link.url.clone(),
                display_name: link.display_name.clone(),
                platform: link.platform.clone(),
            };
            (key.clone(), proto_link)
        })
        .collect()
}

/// Converts security contact to protocol buffer format
fn convert_security_contact(contact: &os_info::SecurityContact) -> osinfo::SecurityContact {
    osinfo::SecurityContact {
        email: contact.email.clone(),
        pgp_key: contact.pgp_key.clone(),
        disclosure_policy: contact.disclosure_policy.clone(),
    }
}
