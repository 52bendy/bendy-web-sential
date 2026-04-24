import { Outlet } from 'react-router-dom';
import { useLayoutStore } from '@/store';
import TopNavbar from './TopNavbar';
import Sidebar from './Sidebar';
import BottomNavbar from './BottomNavbar';

export default function Layout() {
  const { menuPosition, sidebarCollapsed } = useLayoutStore();

  return (
    <div className="min-h-screen bg-[var(--bg-primary)]">
      {/* Top Navigation */}
      {menuPosition === 'top' && <TopNavbar />}

      {/* Left Sidebar */}
      {menuPosition === 'left' && <Sidebar />}

      {/* Main Content */}
      <main
        className={`p-6 transition-all duration-300 ${
          menuPosition === 'left'
            ? sidebarCollapsed ? 'ml-16' : 'ml-64'
            : menuPosition === 'bottom'
            ? 'mb-12'
            : ''
        }`}
      >
        <Outlet />
      </main>

      {/* Bottom Navigation */}
      {menuPosition === 'bottom' && <BottomNavbar />}
    </div>
  );
}
