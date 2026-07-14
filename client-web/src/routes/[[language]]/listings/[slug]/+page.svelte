<script lang="ts">
  import ListingDetailContainer from "$lib/components/listing/ListingDetailContainer.svelte";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import { i18n } from "$lib/i18n.js";
  import { resolveMediaUrl } from "$lib/shared/utils/media.js";
  import { getLocalizedField } from "$lib/utils/localize.js";

  let { data } = $props();
  const listing = $derived(data?.listing);
  const name = $derived(
    listing ? getLocalizedField(listing, "title", getLocale()) : "",
  );
  const description = $derived(
    listing ? getLocalizedField(listing, "description", getLocale()) : "",
  );
  
  function getPlainTextDescription(desc: string) {
    if (!desc) return "";
    try {
      const parsed = JSON.parse(desc);
      if (Array.isArray(parsed)) {
        return parsed
          .filter(b => ["text", "heading", "subheading", "list"].includes(b.type))
          .map(b => b.content || b.contentAr || b.contentEn || "")
          .join(" ")
          .replace(/\s+/g, " ")
          .trim();
      }
    } catch {
      // Return plain text if not JSON
    }
    return desc;
  }

  const plainTextDescription = $derived(
    listing ? getPlainTextDescription(description) : ""
  );

  const metaTitle = $derived(
    listing ? getLocalizedField(listing, "metaTitle", getLocale()) : "",
  );
  const metaDescription = $derived(
    listing ? getLocalizedField(listing, "metaDescription", getLocale()) : "",
  );
  const vendorName = $derived(
    listing ? getLocalizedField(listing.vendor, "name", getLocale()) : "",
  );
</script>

<svelte:head>
  {#if listing}
    <title
      >{metaTitle || `${name} — ${vendorName} | ${m.meta_siteName()}`}</title
    >
    <meta
      name="description"
      content={metaDescription || plainTextDescription?.slice(0, 155) || ""}
    />
    <meta
      property="og:title"
      content={metaTitle || `${name} — ${vendorName}`}
    />
    <meta
      property="og:description"
      content={metaDescription || plainTextDescription?.slice(0, 155) || ""}
    />
    <meta property="og:image" content={resolveMediaUrl(listing.coverImage)} />
    <meta
      property="og:url"
      content={`https://zafafworld.net${i18n.resolveRoute(`/listings/${listing.slug}`, getLocale() as any)}`}
    />
    <meta
      property="og:locale"
      content={getLocale() === "ar" ? "ar_SA" : "en_US"}
    />
    <meta
      property="og:locale:alternate"
      content={getLocale() === "ar" ? "en_US" : "ar_SA"}
    />

    <link
      rel="canonical"
      href={`https://zafafworld.net/listings/${listing.slug}`}
    />
  {:else}
    <title>{m.auto_loading()} | {m.meta_siteName()}</title>
  {/if}
</svelte:head>

<ListingDetailContainer {data} />
