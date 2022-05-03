int main() {
  int arr[10], n = getarray(arr);
  int i = 0, sum = 0;
  while (i < n) {
    sum = sum + arr[i];
    i = i + 1;
  }
  putint(sum);
  putch(10);
  return 0;
}