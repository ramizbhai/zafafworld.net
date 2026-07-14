const fs = require('fs');
const https = require('https');

const API_URL = 'http://localhost:8000/api/v1'; // Running backend directly or via nginx?
// Actually nginx is probably on 80/443, pgbouncer 5432, backend 8000. Wait, UDS is used!
// So it's best to run fetch against nginx, or use curl inside the container.
// Nginx handles requests for api.zafafworld.net, vendor.zafafworld.net etc.
// The easiest way is to use `podman exec -it zafafworld_backend_1 curl ...` OR since nginx is listening on host:
// Let's use `fetch` targeting http://localhost (since nginx is on port 80).
// Wait, the API routes are mapped in Nginx. Let's write a simple script that uses HTTP to localhost, with Host header: `api.zafafworld.net`.

async function run() {
    try {
        console.log("1. Logging in...");
        const loginRes = await fetch("http://localhost/api/v1/auth/login", {
            method: "POST",
            headers: {
                "Host": "api.zafafworld.net",
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                email: "ramiz@zafafworld.net",
                password: "Ramiz@789"
            })
        });

        if (!loginRes.ok) {
            const err = await loginRes.text();
            throw new Error(`Login failed: ${loginRes.status} ${err}`);
        }

        const loginData = await loginRes.json();
        const token = loginData.token;
        console.log("Login successful! Token:", token.substring(0, 20) + "...");

        console.log("2. Creating a new listing (product) with photo and video...");
        const payload = {
            title_en: "Zero Trust Test Listing E2E",
            title_ar: "اختبار القائمة 2",
            base_price_sar: 1500,
            deposit_percentage: 25,
            gender_section: "women_only",
            coordinator_name_en: "E2E Coordinator",
            coordinator_phone: "0500000000",
            city_id: 1, // Assuming Riyadh is 1
            gallery_items: [
                {
                    image_url: "/assets/uploads/test-photo.jpg",
                    is_cover: true,
                    media_type: "image"
                },
                {
                    image_url: "/assets/uploads/test-video.mp4",
                    is_cover: false,
                    media_type: "video"
                }
            ]
        };

        const createRes = await fetch("http://localhost/api/v1/vendor/products", {
            method: "POST",
            headers: {
                "Host": "api.zafafworld.net",
                "Content-Type": "application/json",
                "Authorization": `Bearer ${token}`
            },
            body: JSON.stringify(payload)
        });

        if (!createRes.ok) {
            const err = await createRes.text();
            throw new Error(`Create product failed: ${createRes.status} ${err}`);
        }

        const createData = await createRes.json();
        console.log("Listing created successfully!", createData);
        console.log("Test completed: SUCCESS");
    } catch (e) {
        console.error("Test failed:", e.message);
    }
}

run();
