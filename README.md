# SIEM
A web based Security Information and Event Management

### Setup
Clone Repo
```bash
git clone https://github.com/borelli28/SIEM.git
```

Cd into project
```bash
cd SIEM
```

##### Backend
Setup /backend
```bash
cd backend
```

Create .env in /backend
```bash
echo "SESSION_SECRET_KEY=d4c9b6258fc5a4ab0c8334d95703685ec07d03e19fa1b3dd5c5cdd483c06850f" > .env && \
echo "DATABASE_URL=logs/logs.db" >> .env
```

Start backend server
```bash
cargo run
```

##### Frontend
Setup frontend in a new tab
```bash
cd ../frontend && bun install && bun dev
```
or use NPM
```bash
cd ../frontend && npm install && npm run dev
```

Open browser in http://localhost:3000/register

Use `admin` as username
