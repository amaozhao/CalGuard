import type { Metadata } from 'next';
import '../styles/globals.css';
import { Shell } from '../components/layout/Shell';

export const metadata: Metadata = {
  title: 'CalGuard',
  description: 'Local-first calendar health analysis',
  icons: {
    icon: '/icon.png',
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <Shell>{children}</Shell>
      </body>
    </html>
  );
}
