use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::errors::AppError;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CategorySchema {
    pub category_id: String,
    pub category_group: String,
    pub uses_cultural_settings: bool,
    pub detail_sections: Vec<DetailSection>,
    pub feature_groups: Vec<FeatureGroup>,
    pub step_overrides: HashMap<String, StepOverride>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DetailSection {
    pub section_id: String,
    pub title_en: String,
    pub title_ar: String,
    pub fields: Vec<FieldRef>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FieldRef {
    pub field_id: String,
    pub required: bool,
    pub visible_when: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeatureGroup {
    pub group_id: String,
    pub title_en: String,
    pub title_ar: String,
    pub options: Vec<FeatureOption>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeatureOption {
    pub option_id: String,
    pub title_en: String,
    pub title_ar: String,
    pub input_type: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepOverride {
    pub status: StepStatus,
    pub reason_en: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Active,
    NotApplicable,
    Optional,
}

pub struct RawDbFeature {
    pub id: uuid::Uuid,
    pub name_en: String,
    pub name_ar: String,
    pub category: String,
    pub input_type: String,
}

pub fn get_schema_for_category(
    category_id: &str,
    db_features: &[RawDbFeature],
) -> Result<CategorySchema, AppError> {
    let (category_group, uses_cultural_settings) = match category_id {
        "wedding-palace" | "hotel-venue" | "villa-resort" | "restaurant-event" | "outdoor-garden" | "chalet" => {
            ("venues", true)
        }
        "rooftop-venue" | "private-beach" => {
            ("venues", false)
        }
        "wedding-gown" | "haute-couture" | "abaya-jalabiya" | "groom-attire" => {
            ("fashion", false)
        }
        "hair-makeup" | "beauty-skincare" | "henna-art" => {
            ("beauty", false)
        }
        "photography-video" | "photo-studio" => {
            ("photography", false)
        }
        "catering" | "wedding-cake" | "wedding-sweets" => {
            ("food", false)
        }
        "entertainment-dj" | "zaffa" | "nasheed-band" => {
            ("entertainment", false)
        }
        "wedding-jewelry" | "wedding-gifts" => {
            ("jewelry_gifts", false)
        }
        "wedding-planner" | "khosha-decor" | "flowers-floral" | "wedding-invitation" | "lighting-av" => {
            ("planning_decor", false)
        }
        "wedding-car" => {
            ("transportation", false)
        }
        _ => return Err(AppError::BadRequest(format!("Unsupported category: {}", category_id))),
    };

    // 1. Build detail sections statically based on category type
    let mut detail_sections = Vec::new();

    if ["wedding-palace", "hotel-venue", "villa-resort", "restaurant-event", "outdoor-garden", "chalet"].contains(&category_id) {
        detail_sections.push(DetailSection {
            section_id: "venue_capacity".to_string(),
            title_en: "Section Capacity".to_string(),
            title_ar: "الطاقة الاستيعابية للأقسام".to_string(),
            fields: vec![
                FieldRef { field_id: "men_capacity".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "women_capacity".to_string(), required: false, visible_when: None },
            ],
        });
        detail_sections.push(DetailSection {
            section_id: "venue_setup".to_string(),
            title_en: "Venue Facilities".to_string(),
            title_ar: "تجهيزات ومرافق المكان".to_string(),
            fields: vec![
                FieldRef { field_id: "has_separate_entrances".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "has_audio_link".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "max_events_per_day".to_string(), required: false, visible_when: None },
            ],
        });
    }

    if ["villa-resort", "chalet"].contains(&category_id) {
        detail_sections.push(DetailSection {
            section_id: "villa_specifications".to_string(),
            title_en: "Villa Specifications".to_string(),
            title_ar: "مواصفات الفلا والشاليه".to_string(),
            fields: vec![
                FieldRef { field_id: "weekend_surcharge_sar".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "private_pool".to_string(), required: false, visible_when: None },
            ],
        });
    }

    if category_id == "hotel-venue" {
        detail_sections.push(DetailSection {
            section_id: "hotel_specifications".to_string(),
            title_en: "Hotel Services".to_string(),
            title_ar: "خدمات الفندق".to_string(),
            fields: vec![
                FieldRef { field_id: "in_house_catering".to_string(), required: false, visible_when: None },
            ],
        });
    }

    if category_id == "restaurant-event" {
        detail_sections.push(DetailSection {
            section_id: "restaurant_specifications".to_string(),
            title_en: "Restaurant Dining".to_string(),
            title_ar: "خيارات المطعم".to_string(),
            fields: vec![
                FieldRef { field_id: "private_hall_available".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "family_section".to_string(), required: false, visible_when: None },
            ],
        });
    }

    if ["photography-video", "photo-studio"].contains(&category_id) {
        detail_sections.push(DetailSection {
            section_id: "photography_specifications".to_string(),
            title_en: "Photography Service details".to_string(),
            title_ar: "تفاصيل خدمة التصوير".to_string(),
            fields: vec![
                FieldRef { field_id: "team_size".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "delivery_weeks".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "women_section_coverage".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "drone_available".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "highlight_reel".to_string(), required: false, visible_when: None },
            ],
        });
    }

    if category_id == "catering" {
        detail_sections.push(DetailSection {
            section_id: "catering_specifications".to_string(),
            title_en: "Catering Details".to_string(),
            title_ar: "تفاصيل بوفيه الضيافة".to_string(),
            fields: vec![
                FieldRef { field_id: "min_guests".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "buffet_or_plated".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "halal_certified".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "team_size".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "taste_testing".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "setup_cleanup".to_string(), required: false, visible_when: None },
            ],
        });
    }

    if ["wedding-gown", "haute-couture", "abaya-jalabiya", "groom-attire"].contains(&category_id) {
        detail_sections.push(DetailSection {
            section_id: "fashion_specifications".to_string(),
            title_en: "Fashion Specifications".to_string(),
            title_ar: "مواصفات الأزياء".to_string(),
            fields: vec![
                FieldRef { field_id: "rehearsal_count".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "tailoring_time_days".to_string(), required: false, visible_when: None },
            ],
        });
    }

    if ["hair-makeup", "beauty-skincare"].contains(&category_id) {
        detail_sections.push(DetailSection {
            section_id: "beauty_specifications".to_string(),
            title_en: "Beauty Services".to_string(),
            title_ar: "خدمات التجميل".to_string(),
            fields: vec![
                FieldRef { field_id: "bride_companions_count".to_string(), required: false, visible_when: None },
            ],
        });
    }

    if category_id == "henna-art" {
        detail_sections.push(DetailSection {
            section_id: "henna_specifications".to_string(),
            title_en: "Henna Art".to_string(),
            title_ar: "نقش الحناء".to_string(),
            fields: vec![
                FieldRef { field_id: "address".to_string(), required: true, visible_when: None },
            ],
        });
    }

    if ["entertainment-dj", "zaffa", "nasheed-band"].contains(&category_id) {
        detail_sections.push(DetailSection {
            section_id: "entertainment_specifications".to_string(),
            title_en: "Performance Details".to_string(),
            title_ar: "تفاصيل العرض الموسيقي".to_string(),
            fields: vec![
                FieldRef { field_id: "events_hosted_description".to_string(), required: false, visible_when: None },
                FieldRef { field_id: "preparation_time_hours".to_string(), required: false, visible_when: None },
            ],
        });
    }

    if category_id == "wedding-car" {
        detail_sections.push(DetailSection {
            section_id: "transport_specifications".to_string(),
            title_en: "Vehicle Details".to_string(),
            title_ar: "تفاصيل المركبات".to_string(),
            fields: vec![
                FieldRef { field_id: "vehicle_count".to_string(), required: false, visible_when: None },
            ],
        });
    }

    // 2. Build feature groups dynamically or statically
    let mut feature_groups = Vec::new();

    if ["wedding-palace", "hotel-venue", "villa-resort", "restaurant-event", "outdoor-garden", "chalet"].contains(&category_id) {
        let mut feature_groups_map: HashMap<String, Vec<FeatureOption>> = HashMap::new();
        for f in db_features {
            feature_groups_map
                .entry(f.category.clone())
                .or_default()
                .push(FeatureOption {
                    option_id: f.id.to_string(),
                    title_en: f.name_en.clone(),
                    title_ar: f.name_ar.clone(),
                    input_type: f.input_type.clone(),
                });
        }
        for (category_name, options) in feature_groups_map {
            let (title_en, title_ar) = match category_name.as_str() {
                "General Info" => ("General Info".to_string(), "معلومات عامة".to_string()),
                "Amenities" => ("Amenities".to_string(), "المرافق والخدمات".to_string()),
                _ => (category_name.clone(), category_name.clone()),
            };
            feature_groups.push(FeatureGroup {
                group_id: category_name.to_lowercase().replace(' ', "_"),
                title_en,
                title_ar,
                options,
            });
        }
    } else if ["wedding-gown", "haute-couture", "abaya-jalabiya", "groom-attire"].contains(&category_id) {
        feature_groups.push(FeatureGroup {
            group_id: "dress_type".to_string(),
            title_en: "Dress Type".to_string(),
            title_ar: "نوع الفستان".to_string(),
            options: vec![
                FeatureOption { option_id: "ball_gown".to_string(), title_en: "Ball Gown".to_string(), title_ar: "فستان منفوش".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "mermaid".to_string(), title_en: "Mermaid".to_string(), title_ar: "فستان سمكة".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "a_line".to_string(), title_en: "A-Line".to_string(), title_ar: "فستان مستقيم (A-Line)".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "custom_design".to_string(), title_en: "Custom Design".to_string(), title_ar: "تصميم خاص".to_string(), input_type: "boolean".to_string() },
            ],
        });
        feature_groups.push(FeatureGroup {
            group_id: "accessories".to_string(),
            title_en: "Accessories".to_string(),
            title_ar: "الإكسسوارات".to_string(),
            options: vec![
                FeatureOption { option_id: "veil".to_string(), title_en: "Veil".to_string(), title_ar: "طرحة".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "crown".to_string(), title_en: "Crown".to_string(), title_ar: "تاج".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "hair_accessories".to_string(), title_en: "Hair Accessories".to_string(), title_ar: "إكسسوارات شعر".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "gloves".to_string(), title_en: "Gloves".to_string(), title_ar: "قفازات".to_string(), input_type: "boolean".to_string() },
            ],
        });
    } else if ["hair-makeup", "beauty-skincare"].contains(&category_id) {
        feature_groups.push(FeatureGroup {
            group_id: "services".to_string(),
            title_en: "Services Offered".to_string(),
            title_ar: "الخدمات المقدمة".to_string(),
            options: vec![
                FeatureOption { option_id: "home_service".to_string(), title_en: "Home Service".to_string(), title_ar: "خدمة منزلية".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "massage".to_string(), title_en: "Massage".to_string(), title_ar: "مساج".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "makeup".to_string(), title_en: "Makeup".to_string(), title_ar: "مكياج".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "hair".to_string(), title_en: "Hair Styling".to_string(), title_ar: "تسريح شعر".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "pedicure".to_string(), title_en: "Pedicure".to_string(), title_ar: "بديكير".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "facial".to_string(), title_en: "Facial Care".to_string(), title_ar: "العناية بالبشرة".to_string(), input_type: "boolean".to_string() },
            ],
        });
    } else if category_id == "henna-art" {
        feature_groups.push(FeatureGroup {
            group_id: "henna_style".to_string(),
            title_en: "Henna Style".to_string(),
            title_ar: "نمط نقش الحناء".to_string(),
            options: vec![
                FeatureOption { option_id: "saudi".to_string(), title_en: "Saudi Style".to_string(), title_ar: "نقش سعودي".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "khaleeji".to_string(), title_en: "Khaleeji".to_string(), title_ar: "نقش خليجي".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "indian".to_string(), title_en: "Indian".to_string(), title_ar: "نقش هندي".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "emirati".to_string(), title_en: "Emirati".to_string(), title_ar: "نقش إماراتي".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "sudanese".to_string(), title_en: "Sudanese".to_string(), title_ar: "نقش سوداني".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "custom".to_string(), title_en: "Custom Pattern".to_string(), title_ar: "نقش مخصص".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "white_henna".to_string(), title_en: "White Henna".to_string(), title_ar: "حنا بيضاء".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "red_henna".to_string(), title_en: "Red Henna".to_string(), title_ar: "حنا حمراء".to_string(), input_type: "boolean".to_string() },
            ],
        });
    } else if ["wedding-jewelry", "wedding-gifts"].contains(&category_id) {
        feature_groups.push(FeatureGroup {
            group_id: "jewelry_type".to_string(),
            title_en: "Jewelry Type & Services".to_string(),
            title_ar: "نوع المجوهرات والخدمات".to_string(),
            options: vec![
                FeatureOption { option_id: "rings".to_string(), title_en: "Rings".to_string(), title_ar: "خواتم".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "diamond".to_string(), title_en: "Diamond Jewelry".to_string(), title_ar: "مجوهرات ألماس".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "bracelets".to_string(), title_en: "Bracelets".to_string(), title_ar: "أساور".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "necklace".to_string(), title_en: "Necklace".to_string(), title_ar: "قلادة".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "custom_design".to_string(), title_en: "Custom Design".to_string(), title_ar: "تصميم خاص".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "warranty".to_string(), title_en: "Certificate of Warranty".to_string(), title_ar: "شهادة ضمان".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "engraving".to_string(), title_en: "Custom Engraving".to_string(), title_ar: "حفر مخصص".to_string(), input_type: "boolean".to_string() },
            ],
        });
    } else if ["entertainment-dj", "zaffa", "nasheed-band"].contains(&category_id) {
        feature_groups.push(FeatureGroup {
            group_id: "entertainment_type".to_string(),
            title_en: "Entertainment Setup".to_string(),
            title_ar: "نوع الترفيه والتجهيزات".to_string(),
            options: vec![
                FeatureOption { option_id: "dj".to_string(), title_en: "DJ".to_string(), title_ar: "دي جي".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "singer".to_string(), title_en: "Live Singer".to_string(), title_ar: "مطرب".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "zaffa_procession".to_string(), title_en: "Zaffa Procession".to_string(), title_ar: "فرقة زفة".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "band".to_string(), title_en: "Band".to_string(), title_ar: "فرقة موسيقية كاملة".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "half_band".to_string(), title_en: "Half Band".to_string(), title_ar: "فرقة مصغرة".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "full_band".to_string(), title_en: "Full Band".to_string(), title_ar: "تخت كامل".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "music_systems".to_string(), title_en: "Audio & Music Systems".to_string(), title_ar: "أنظمة صوتية وموسيقية".to_string(), input_type: "boolean".to_string() },
            ],
        });
    } else if category_id == "wedding-car" {
        feature_groups.push(FeatureGroup {
            group_id: "vehicle_type".to_string(),
            title_en: "Vehicle Type".to_string(),
            title_ar: "نوع السيارة".to_string(),
            options: vec![
                FeatureOption { option_id: "limousine".to_string(), title_en: "Limousine".to_string(), title_ar: "ليموزين".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "classic_car".to_string(), title_en: "Classic Car".to_string(), title_ar: "سيارة كلاسيكية".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "luxury_suv".to_string(), title_en: "Luxury SUV".to_string(), title_ar: "سيارة دفع رباعي فاخرة".to_string(), input_type: "boolean".to_string() },
                FeatureOption { option_id: "decorated_car".to_string(), title_en: "Decorated Wedding Car".to_string(), title_ar: "سيارة زفاف مزينة".to_string(), input_type: "boolean".to_string() },
            ],
        });
    }

    // 3. Build step overrides
    let mut step_overrides = HashMap::new();
    if !uses_cultural_settings {
        step_overrides.insert(
            "4".to_string(),
            StepOverride {
                status: StepStatus::NotApplicable,
                reason_en: Some("Cultural settings are only applicable for venue listings.".to_string()),
            },
        );
    } else {
        step_overrides.insert(
            "4".to_string(),
            StepOverride {
                status: StepStatus::Active,
                reason_en: None,
            },
        );
    }

    Ok(CategorySchema {
        category_id: category_id.to_string(),
        category_group: category_group.to_string(),
        uses_cultural_settings,
        detail_sections,
        feature_groups,
        step_overrides,
    })
}
