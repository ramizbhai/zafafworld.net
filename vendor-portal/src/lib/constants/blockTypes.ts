import {
    Heading,
    Heading2,
    AlignLeft,
    List,
    Image as ImageIcon,
    Images,
    Video,
    MapPin,
    Link as LinkIcon,
    Phone,
    Share2,
    Minus,
} from "lucide-svelte";

export const BLOCK_TYPES = [
    {
        id: "heading",
        labelAr: "عنوان رئيسي",
        labelEn: "Heading",
        icon: Heading,
        group: "text",
    },
    {
        id: "subheading",
        labelAr: "عنوان فرعي",
        labelEn: "Sub Heading",
        icon: Heading2,
        group: "text",
    },
    {
        id: "text",
        labelAr: "فقرة / وصف",
        labelEn: "Paragraph",
        icon: AlignLeft,
        group: "text",
    },
    {
        id: "list",
        labelAr: "قائمة",
        labelEn: "List",
        icon: List,
        group: "text",
    },
    {
        id: "image",
        labelAr: "صورة",
        labelEn: "Image",
        icon: ImageIcon,
        group: "media",
    },
    {
        id: "gallery",
        labelAr: "معرض صور",
        labelEn: "Gallery",
        icon: Images,
        group: "media",
    },
    {
        id: "video",
        labelAr: "فيديو",
        labelEn: "Video",
        icon: Video,
        group: "media",
    },
    {
        id: "map",
        labelAr: "خريطة",
        labelEn: "Map",
        icon: MapPin,
        group: "media",
    },
    {
        id: "button",
        labelAr: "زر رابط",
        labelEn: "Button Link",
        icon: LinkIcon,
        group: "interactive",
    },
    {
        id: "contact",
        labelAr: "معلومات تواصل",
        labelEn: "Contact Info",
        icon: Phone,
        group: "interactive",
    },
    {
        id: "social",
        labelAr: "روابط سوشيال",
        labelEn: "Social Links",
        icon: Share2,
        group: "interactive",
    },
    {
        id: "divider",
        labelAr: "فاصل",
        labelEn: "Divider",
        icon: Minus,
        group: "structure",
    },
];

export function getBlockDef(type: string) {
    return BLOCK_TYPES.find((t) => t.id === type) || BLOCK_TYPES[2];
}
