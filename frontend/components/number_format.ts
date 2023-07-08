const numberFormat = new Intl.NumberFormat();

export function formatNumber(n?: number) {
  if (!n) {
    return "";
  }
  return numberFormat.format(n);
}
