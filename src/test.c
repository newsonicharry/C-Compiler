
void other(int *p2) {
  *p2 *= 2;
  return;
}

int main() {
  int x = 42;
  int *p1 = &x;

  other(p1);

  return *p1;
}
