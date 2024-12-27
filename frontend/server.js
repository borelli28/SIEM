const server = Bun.serve({
    async fetch(req) {
        const path = new URL(req.url).pathname;

        // Route to serve HTML files
        if (path === "/") return new Response(Bun.file("public/index.html"));
        if (path === "/login.html") return new Response(Bun.file("public/login.html"));
        if (path === "/register.html") return new Response(Bun.file("public/register.html"));
        if (path === "/settings.html") return new Response(Bun.file("public/settings.html"));
        if (path === "/search.html") return new Response(Bun.file("public/search.html"));
        if (path === "/alerts.html") return new Response(Bun.file("public/alerts.html"));

        // Serve static CSS and JS files
        if (path.startsWith("/css/")) return new Response(Bun.file(`public${path}`));
        if (path.startsWith("/js/")) return new Response(Bun.file(`public${path}`));
        
        // Handle API calls
        if (path === "/api") return Response.json({ some: "buns", for: "you" });

        // 404
        return new Response("Page not found", { status: 404 });
    },
});

console.log(`Listening on ${server.url}`);