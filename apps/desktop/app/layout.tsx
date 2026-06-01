import type { Metadata } from 'next';
import '../styles/globals.css';
import { Shell } from '../components/layout/Shell';
import { LanguageProvider } from '../lib/i18n';

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
        <LanguageProvider>
          <Shell>{children}</Shell>
        </LanguageProvider>
      </body>
    </html>
  );
}
