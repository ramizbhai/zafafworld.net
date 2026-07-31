import type { PageServerLoad } from './$types';
import { apiClient } from '$lib/services/api/client.js';
import { MockBlogRepository } from '$lib/lib/blog/BlogRepository';

export const load: PageServerLoad = async ({ fetch: svelteFetch, cookies }) => {
  const country = cookies.get('zafaf_selected_country') || 'sa';

  let listings: any[] = [];
  try {
    const seed = Math.floor(Math.random() * 1000000);
    const listingsRes = await apiClient.get<any>(
      `/api/v1/public/listings?tier=diamond&limit=5&seed=${seed}`,
      { fetch: svelteFetch, isServer: true }
    );
    if (listingsRes && listingsRes.listings) {
      listings = listingsRes.listings;
    }
  } catch (err) {
    console.error('[Preview Server Loader] Failed to fetch featured services:', err);
  }

  // Fetch testimonials
  let testimonials: any[] = [];
  try {
    const testimonialsRes = await apiClient.get<any>(
      '/api/v1/public/testimonials',
      { fetch: svelteFetch, isServer: true }
    ).catch(() => null);
    if (testimonialsRes && testimonialsRes.status === 'success') {
      testimonials = testimonialsRes.data || [];
    }
  } catch (err) {
    console.error('[Preview Server Loader] Failed to fetch testimonials:', err);
  }

  // Fetch blog posts using repository adapter
  let blogPosts: any[] = [];
  try {
    const blogRepo = new MockBlogRepository();
    blogPosts = await blogRepo.getLatestPosts(3);
  } catch (err) {
    console.error('[Preview Server Loader] Failed to fetch blog posts:', err);
  }

  // Fetch categories and cities in parallel
  let categories: any[] = [];
  let cities: any[] = [];
  try {
    const [catRes, cityRes] = await Promise.all([
      apiClient.get<any>('/api/v1/public/categories', { fetch: svelteFetch, isServer: true }).catch(() => null),
      apiClient.get<any>('/api/v1/public/cities', { fetch: svelteFetch, isServer: true }).catch(() => null)
    ]);

    if (catRes) {
      let flattened: any[] = [];
      if (Array.isArray(catRes.allCategories)) {
        flattened = catRes.allCategories;
      } else if (catRes.categories) {
        const venuesList = catRes.categories.venues || [];
        const servicesList = catRes.categories.services || [];
        flattened = [...venuesList, ...servicesList];
      }

      categories = flattened.map((c: any) => ({
        key: c.slug,
        icon: '✨',
        labelAr: c.ar,
        labelEn: c.en,
        listingsCount: c.listingsCount ?? 0
      }));
    }

    if (cityRes) {
      const rawCities = cityRes.cities || cityRes.data || [];
      cities = rawCities.map((c: any) => ({
        id: c.id,
        slug: c.slug,
        name_ar: c.name_ar || c.ar || '',
        name_en: c.name_en || c.en || '',
        country_id: c.country_id || ''
      }));
    }
  } catch (err) {
    console.error('[Preview Server Loader] Failed to fetch categories/cities:', err);
  }

  // Fallbacks for local offline development
  if (listings.length === 0) {
    listings = [
      {
        slug: "luxury-royal-ballroom",
        title_en: "The Royal Ballroom at Al-Nakhil",
        title_ar: "القاعة الملكية في النخيل",
        cover_image: "/categories/wedding-palace.webp",
        description_en: "An exquisite grand hall featuring classical architecture, crystal chandeliers, and premium catering services for up to 800 guests.",
        description_ar: "قاعة أفراح فاخرة تتميز بالهندسة المعمارية الكلاسيكية، الثريات الكريستالية، وخدمات الضيافة المتميزة التي تتسع لـ 800 ضيف.",
        price: "15,000",
        category: { en: "Wedding Palace", ar: "قاعة أفراح" },
        city: { name_en: "Riyadh", name_ar: "الرياض" }
      },
      {
        slug: "sunset-gardens-resort",
        title_en: "Sunset Gardens Luxury Resort",
        title_ar: "منتجع حدائق الغروب الفاخر",
        cover_image: "/categories/outdoor-garden.webp",
        description_en: "Beautiful outdoor garden venue overlooking natural ponds, perfect for elegant evening engagement parties and boutique receptions.",
        description_ar: "مكان خارجي جميل مطل على بحيرات طبيعية، مثالي لحفلات الخطوبة المسائية الأنيقة والاستقبالات الخاصة.",
        price: "8,500",
        category: { en: "Villa & Resort", ar: "استراحات وفلل" },
        city: { name_en: "Jeddah", name_ar: "جدة" }
      },
      {
        slug: "elite-floral-designs",
        title_en: "Elite Floral & Event Stylists",
        title_ar: "نخبة لتصميم وتنسيق المناسبات",
        cover_image: "/categories/wedding-planner.webp",
        description_en: "Award-winning floral arrangements, stage setups, and customized wedding decorations tailored to turn your vision into reality.",
        description_ar: "تنسيقات زهور حائزة على جوائز، تجهيز منصات الكوشة، وديكورات زفاف مخصصة لتحويل رؤيتك إلى واقع ملموس.",
        price: "12,000",
        category: { en: "Wedding Planner", ar: "منظم حفلات" },
        city: { name_en: "Dammam", name_ar: "الدمام" }
      }
    ];
  }

  if (categories.length === 0) {
    categories = [
      { key: "wedding-palace", slug: "wedding-palace", icon: "🏛️", labelAr: "قاعات الأفراح", labelEn: "Wedding Palace" },
      { key: "hotel-venue", slug: "hotel-venue", icon: "🏨", labelAr: "فنادق وقاعات", labelEn: "Hotel Ballroom" },
      { key: "villa-resort", slug: "villa-resort", icon: "🏡", labelAr: "استراحات وفلل", labelEn: "Villa & Resort" },
      { key: "wedding-planner", slug: "wedding-planner", icon: "📋", labelAr: "منظم حفلات", labelEn: "Wedding Planner" }
    ];
  }

  if (cities.length === 0) {
    cities = [
      { id: "riyadh", slug: "riyadh", name_ar: "الرياض", name_en: "Riyadh", country_id: "sa" },
      { id: "jeddah", slug: "jeddah", name_ar: "جدة", name_en: "Jeddah", country_id: "sa" },
      { id: "dammam", slug: "dammam", name_ar: "الدمام", name_en: "Dammam", country_id: "sa" }
    ];
  }
  if (testimonials.length === 0) {
    testimonials = [
      {
        id: 1,
        name: { en: "Sarah & Fahad", ar: "سارة وفهد" },
        text: { 
          en: "ZafafWorld made our wedding coordination absolutely seamless! The palace booking was verified in 24 hours, and the customer service helped us select the perfect floral designer.",
          ar: "لقد جعلت منصة زفاف وورلد تخطيط زفافنا سهلاً وممتعاً للغاية! تم تأكيد حجز القاعة في غضون ٢٤ ساعة، وساعدنا فريق الدعم في اختيار أفضل منسق زهور."
        },
        image: "/categories/hair-makeup.webp",
        rating: 5,
        city: { name: { en: "Riyadh", ar: "الرياض" } },
        weddingDate: "October 2025",
        vendorUsed: "The Royal Ballroom"
      },
      {
        id: 2,
        name: { en: "Reem & Khalid", ar: "ريم وخالد" },
        text: { 
          en: "Outstanding experience! The site helped us filter venues by budget and city. We saved 15% on booking fees through the Zafaf discount package.",
          ar: "تجربة استثنائية! ساعدنا الموقع في تصفية القاعات حسب الميزانية والمدينة. ووفرنا ما يقارب ١٥٪ من رسوم الحجز الإضافية عبر باقات خصم المنصة."
        },
        image: "/categories/haute-couture.webp",
        rating: 5,
        city: { name: { en: "Jeddah", ar: "جدة" } },
        weddingDate: "November 2025",
        vendorUsed: "Sunset Resort Foyer"
      },
      {
        id: 3,
        name: { en: "Amna & Faisal", ar: "آمنة وفيصل" },
        text: { 
          en: "The best directory for Gulf weddings. Every listing has authentic images and clear layouts. The concierge team is highly responsive.",
          ar: "الدليل الأفضل لمستلزمات الحفلات في الخليج. كل عرض يحتوي على صور واقعية ومخططات واضحة. فريق تنسيق الأفراح يستحق كل الشكر والثناء."
        },
        image: "/categories/beauty-skincare.webp",
        rating: 5,
        city: { name: { en: "Dubai", ar: "دبي" } },
        weddingDate: "December 2025",
        vendorUsed: "Private Beach Pier"
      }
    ];
  }

  return {
    listings,
    categories,
    cities,
    testimonials,
    blogPosts,
    selectedCountry: country
  };
};
