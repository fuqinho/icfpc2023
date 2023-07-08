const numberFormat = new Intl.NumberFormat();

export function formatNumber(n?: number) {
  if (n === undefined) {
    return "";
  }
  return numberFormat.format(n);
}
