
int pow(int x, int n) {
  int sum = 1;
  while (n != 0) {
    if (n % 2) sum = sum * x;
    x = x * x;
    n = n / 2;
  }
  return sum;
}

int main()
{
  int n = 10;
  int x = 2;
  putint(pow(x, n));
  return 0;
}