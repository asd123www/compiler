int f1(int a0[], int a1[], int a2[], int a3[], int a4[]) {
  return a0[0] + a1[1] + a2[2] + a3[3] + a4[4];
}

int main() {
  int arr[10][10][10];
  f1(arr[0][0], arr[1][1], arr[2][2], arr[3][3], arr[4][4]);
  return 0;
}