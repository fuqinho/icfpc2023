import NavBar from "./NavBar";
import "./globals.css";

export const metadata = {
  title: "Dashboard",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="ja">
      <body>
        <NavBar />
        {children}
      </body>
    </html>
  );
}
