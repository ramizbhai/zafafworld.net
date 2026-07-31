import { env } from '$env/dynamic/public';

export interface BlogPost {
  id: string | number;
  title: string;
  excerpt: string;
  slug: string;
  cover_image: string;
  category: {
    en: string;
    ar: string;
  };
  reading_time: {
    en: string;
    ar: string;
  };
  publish_date: {
    en: string;
    ar: string;
  };
}

export interface BlogRepository {
  /**
   * Fetches the latest blog posts.
   * 
   * @param limit Maximum number of blog posts to return.
   */
  getLatestPosts(limit: number): Promise<BlogPost[]>;
}

export class ProductionBlogRepository implements BlogRepository {
  private apiBase = env.PUBLIC_API_URL || 'http://localhost:8080';

  async getLatestPosts(limit: number): Promise<BlogPost[]> {
    try {
      const response = await fetch(`${this.apiBase}/api/v1/public/blogs?limit=${limit}`);
      if (!response.ok) {
        throw new Error(`HTTP error ${response.status}`);
      }
      const json = await response.json();
      if (json && Array.isArray(json.data)) {
        return json.data.slice(0, limit).map((b: any) => ({
          id: b.id || b.slug,
          title: b.title || "",
          excerpt: b.excerpt || b.summary || "",
          slug: b.slug || "",
          cover_image: b.cover_image || b.image || "/categories/wedding-palace.webp",
          category: {
            en: b.category?.en || "Planning Guide",
            ar: b.category?.ar || "دليل التخطيط"
          },
          reading_time: {
            en: b.reading_time?.en || "5 min read",
            ar: b.reading_time?.ar || "5 دقائق قراءة"
          },
          publish_date: {
            en: b.publish_date?.en || "July 2026",
            ar: b.publish_date?.ar || "يوليو ٢٠٢٦"
          }
        }));
      }
      return [];
    } catch (err) {
      console.warn('[ProductionBlogRepository] Fetch error, falling back to mock data:', err);
      const fallback = new MockBlogRepository();
      return fallback.getLatestPosts(limit);
    }
  }
}

export class MockBlogRepository implements BlogRepository {
  async getLatestPosts(limit: number): Promise<BlogPost[]> {
    // Return SSR-safe mocked blog data in bilingual format
    const mockPosts: BlogPost[] = [
      {
        id: 1,
        title: "أفضل 10 قاعات أفراح فاخرة في الرياض لعام 2026",
        excerpt: "اكتشف قائمتنا المختارة بعناية لأكثر قاعات وقصور الأفراح فخامة وتميزاً في العاصمة الرياض مع تفاصيل الأسعار والخدمات والمساحات المتوفرة.",
        slug: "top-10-luxury-wedding-halls-riyadh",
        cover_image: "/categories/wedding-palace.webp",
        category: {
          en: "Planning Guide",
          ar: "دليل التخطيط"
        },
        reading_time: {
          en: "5 min read",
          ar: "5 دقائق قراءة"
        },
        publish_date: {
          en: "July 28, 2026",
          ar: "٢٨ يوليو ٢٠٢٦"
        }
      },
      {
        id: 2,
        title: "دليلك الكامل لتنسيق ألوان زفاف صيفي أنيق",
        excerpt: "تخطط لحفل زفاف صيفي؟ إليك أحدث اتجاهات باليتات الألوان المبهجة والمتناغمة التي تمنح زفافك طابعاً منعشاً وفخماً في آن واحد.",
        slug: "complete-guide-summer-wedding-color-palette",
        cover_image: "/categories/flowers-floral.webp",
        category: {
          en: "Inspiration",
          ar: "أفكار ملهمة"
        },
        reading_time: {
          en: "4 min read",
          ar: "٤ دقائق قراءة"
        },
        publish_date: {
          en: "July 25, 2026",
          ar: "٢٥ يوليو ٢٠٢٦"
        }
      },
      {
        id: 3,
        title: "كيف تختار منظم الزفاف المناسب لميزانيتك؟",
        excerpt: "خطوات عملية وأسئلة جوهرية لطرحها على منسق الحفلات قبل التعاقد، لتضمن الحصول على زفاف أحلامك دون تخطي الحدود المالية المحددة.",
        slug: "how-to-choose-wedding-planner-budget",
        cover_image: "/categories/wedding-planner.webp",
        category: {
          en: "Expert Advice",
          ar: "نصائح الخبراء"
        },
        reading_time: {
          en: "6 min read",
          ar: "٦ دقائق قراءة"
        },
        publish_date: {
          en: "July 20, 2026",
          ar: "٢٠ يوليو ٢٠٢٦"
        }
      }
    ];

    return mockPosts.slice(0, limit);
  }
}
