const numberFormat = new Intl.NumberFormat();

export function formatNumber(n?: number) {
  if (n === undefined) {
    return "";
  }
  return numberFormat.format(n);
}

export function formatPercentage(n?: number) {
  if (n === undefined) {
    return "";
  }
  return n.toLocaleString(undefined, {
    style: "percent",
    minimumFractionDigits: 2,
  });
}
