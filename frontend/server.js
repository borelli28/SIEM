const server = Bun.serve({
    async fetch(req) {
        const path = new URL(req.url).pathname;

        // Serve HTML files
        if (path === "/") return new Response(Bun.file("public/index.html"));
        if (path === "/login") return new Response(Bun.file("public/login.html"));
        if (path === "/register") return new Response(Bun.file("public/register.html"));
        if (path === "/settings") return new Response(Bun.file("public/settings.html"));
        if (path === "/search") return new Response(Bun.file("public/search.html"));
        if (path === "/alerts") return new Response(Bun.file("public/alerts.html"));
        if (path === "/cases") return new Response(Bun.file("public/cases.html"));
        if (path === "/list-cases") return new Response(Bun.file("public/list-cases.html"));

        // Serve static CSS and JS files
        if (path.startsWith("/css/")) return new Response(Bun.file(`public${path}`));
        if (path.startsWith("/js/")) return new Response(Bun.file(`public${path}`));

        // Serve service files
        if (path.startsWith("/services/")) {
            const filePath = `services${path.substring("/services".length)}`;
            try {
                return new Response(Bun.file(filePath));
            } catch (error) {
                console.error(`Error serving ${filePath}:`, error);
                return new Response("File not found", { status: 404 });
            }
        }

        // Handle API calls (example)
        if (path === "/api") return Response.json({ some: "buns", for: "you" });
        
        // 404
        return new Response("Page not found", { status: 404 });
    },
});

console.log(`Listening on ${server.url}`);