---
name: frontend-react
description: Frontend development for bendy-web-sential admin UI. Use when working on frontend/ directory, React components, pages, API calls, i18n, or styling.
when_to_use: Adding new pages, components, API integration, internationalization, UI improvements.
---

# Frontend React Development Guide

## Project Structure
```
frontend/
├── src/
│   ├── main.tsx           # Entry point
│   ├── App.tsx            # Root component
│   ├── pages/             # Page components
│   │   ├── Dashboard.tsx
│   │   ├── Domains.tsx
│   │   ├── Routes.tsx
│   │   ├── AuditLog.tsx
│   │   ├── Settings.tsx
│   │   ├── Login.tsx
│   │   └── RouteTest.tsx
│   ├── components/        # Reusable components
│   │   ├── Layout.tsx
│   │   ├── Sidebar.tsx
│   │   ├── TopNavbar.tsx
│   │   ├── BottomNavbar.tsx
│   │   └── TrafficChart.tsx
│   ├── lib/
│   │   ├── api.ts         # API client
│   │   └── utils.ts       # Utility functions
│   ├── store/             # State management
│   └── i18n/              # Internationalization
│       ├── index.ts
│       ├── en.ts
│       └── zh.ts
├── tailwind.config.js     # Tailwind CSS config
├── vite.config.ts         # Vite config
└── package.json
```

## Tech Stack
- **Framework**: React 18 + TypeScript
- **Build Tool**: Vite
- **Styling**: Tailwind CSS + shadcn/ui
- **State**: Zustand (store/index.ts)
- **i18n**: i18next
- **HTTP**: Fetch API (via lib/api.ts)

## API Integration
API base URL: Backend Admin API at port 3000

Use the unified API client in `lib/api.ts`:
```typescript
import { api } from '@/lib/api';
// All API calls go through this unified client
```

## API Response Format
```json
{"code": 0, "message": "ok", "data": null}
```

Handle errors with unified error handling in API client.

## i18n Usage
```typescript
import { useTranslation } from 'react-i18next';
const { t } = useTranslation();
// Use t('key') for translations
```

## UI Theme
- Black/white/gray color scheme
- Light/dark mode toggle in TopNavbar
- Use Tailwind CSS utilities
- Follow shadcn/ui component patterns

## Commands
```bash
cd frontend
npm run dev          # Development server
npm run build       # Production build
npm run preview      # Preview production build
```
