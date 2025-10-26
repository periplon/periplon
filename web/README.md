# DSL Executor Web UI

Modern web interface for the DSL Executor workflow orchestration platform.

## Features

- 📊 **Dashboard** - Real-time metrics and overview
- 🔄 **Workflows** - Create, edit, and manage workflow definitions
- ▶️ **Executions** - Monitor workflow runs with real-time updates
- ⏰ **Schedules** - Configure cron-based workflow scheduling
- 🔑 **API Keys** - Manage programmatic access credentials
- 🔐 **Authentication** - Secure JWT-based authentication
- 🎨 **Modern UI** - Built with Next.js 14, Tailwind CSS, and shadcn/ui

## Tech Stack

- **Framework**: Next.js 14 (App Router)
- **Language**: TypeScript
- **Styling**: Tailwind CSS
- **UI Components**: shadcn/ui (Radix UI)
- **State Management**: Zustand
- **Server State**: TanStack Query (React Query)
- **HTTP Client**: Axios
- **Icons**: Lucide React
- **Forms**: React Hook Form + Zod

## Getting Started

### Prerequisites

- Node.js 18+ installed
- DSL Executor server running (default: http://localhost:8080)

### Installation

```bash
# Install dependencies
npm install

# Set up environment variables
cp .env.example .env.local

# Edit .env.local with your API URL
# NEXT_PUBLIC_API_URL=http://localhost:8080
```

### Development

```bash
# Start development server
npm run dev

# Open http://localhost:3000 in your browser
```

### Build

```bash
# Build for production
npm run build

# Start production server
npm start
```

## Environment Variables

```env
NEXT_PUBLIC_API_URL=http://localhost:8080
```

## Project Structure

```
web/
├── src/
│   ├── app/                    # Next.js app router pages
│   │   ├── (auth)/            # Authentication pages (login)
│   │   ├── (dashboard)/       # Dashboard pages (protected)
│   │   │   ├── dashboard/
│   │   │   ├── workflows/
│   │   │   ├── executions/
│   │   │   ├── schedules/
│   │   │   └── settings/
│   │   ├── layout.tsx         # Root layout
│   │   └── globals.css        # Global styles
│   ├── components/
│   │   ├── ui/                # Reusable UI components
│   │   ├── layout/            # Layout components (Sidebar)
│   │   └── providers/         # React providers
│   ├── lib/
│   │   ├── api-client.ts      # API client with Axios
│   │   └── utils.ts           # Utility functions
│   ├── stores/
│   │   └── auth-store.ts      # Authentication state (Zustand)
│   ├── types/
│   │   └── index.ts           # TypeScript type definitions
│   └── hooks/                 # Custom React hooks
├── package.json
├── tsconfig.json
├── tailwind.config.ts
└── next.config.js
```

## Key Features Detail

### Dashboard
- Real-time metrics display
- Active executions count
- Success rate tracking
- Recent executions list
- Quick action buttons

### Workflows
- Grid/list view of all workflows
- Version management
- YAML/JSON editor
- Workflow validation
- Tags and metadata
- Execution trigger
- Natural language generation (future)

### Executions
- Real-time status updates (5s polling)
- Execution logs viewer
- Cancel running executions
- Retry failed executions
- Filter by status/date
- Duration tracking

### Schedules
- Cron expression configuration
- Calendar view (future)
- Manual trigger
- Next run preview
- Schedule history

### Settings
- API key management
- Key rotation
- Scoped permissions
- Expiration dates
- Usage tracking
- User profile management

## API Integration

The web UI communicates with the DSL Executor REST API:

- **Base URL**: `http://localhost:8080/api/v1`
- **Authentication**: JWT Bearer tokens
- **Auto-refresh**: Automatic token refresh on 401
- **Error Handling**: Centralized error interceptors

## Development

### Adding New Pages

1. Create page in `src/app/(dashboard)/[page-name]/page.tsx`
2. Add route to sidebar navigation in `src/components/layout/sidebar.tsx`
3. Create API methods in `src/lib/api-client.ts`
4. Define types in `src/types/index.ts`

### Adding New Components

```bash
# Create in src/components/ui/
# Follow shadcn/ui patterns
# Use cn() utility for class merging
```

### State Management

- **Server State**: Use TanStack Query for API data
- **Client State**: Use Zustand for local state
- **Auth State**: Persisted in Zustand with localStorage

## Deployment

### Vercel (Recommended)

```bash
# Install Vercel CLI
npm i -g vercel

# Deploy
vercel
```

### Docker

```bash
# Build image
docker build -t dsl-executor-ui .

# Run container
docker run -p 3000:3000 -e NEXT_PUBLIC_API_URL=http://api:8080 dsl-executor-ui
```

### Static Export

```bash
# Add to next.config.js:
# output: 'export'

npm run build
# Static files in ./out/
```

## Contributing

1. Fork the repository
2. Create feature branch
3. Make changes
4. Run linting: `npm run lint`
5. Type check: `npm run type-check`
6. Submit pull request

## License

Same as parent project (MIT OR Apache-2.0)
