#!/bin/bash
set -e

echo "1. Logging in..."
TOKEN=$(curl -k -s -X POST https://localhost/api/v1/auth/login \
  -H "Host: api.zafafworld.net" \
  -H "Content-Type: application/json" \
  -d '{"email": "ramiz@zafafworld.net", "password": "Ramiz@789", "domain_type": "Vendor"}' | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo "Login failed. Could not extract token."
    exit 1
fi
echo "Login successful. Token: ${TOKEN:0:20}..."

echo "2. Creating product..."
PAYLOAD=$(cat <<EOF
{
    "titleEn": "Zero Trust Test Listing E2E",
    "titleAr": "اختبار القائمة 2",
    "basePriceSar": 1500,
    "depositPercentage": 25,
    "genderSection": "women_only",
    "coordinatorNameEn": "E2E Coordinator",
    "coordinatorNameAr": "منسق E2E",
    "coordinatorPhone": "0500000000",
    "coordinatorWhatsapp": "0500000000",
    "coordinatorEmail": "e2e@example.com",
    "cityId": "a3f9b2d8-1c4e-4b2a-8f5d-7a6c9e0b1c2d",
    "galleryItems": [
        {
            "imageUrl": "https://example.com/test-photo.jpg",
            "isCover": true,
            "mediaType": "image"
        },
        {
            "imageUrl": "https://example.com/test-video.mp4",
            "isCover": false,
            "mediaType": "video"
        }
    ]
}
EOF
)

RESPONSE=$(curl -k -s -w "\nHTTP_STATUS:%{http_code}" -X POST https://localhost/api/v1/vendor/products \
  -H "Host: api.zafafworld.net" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "$PAYLOAD")

echo "$RESPONSE"
