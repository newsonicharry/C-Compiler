


#include <stdio.h>
typedef struct Point{
 int x;
 int y;
} Point;




int main(){
 Point my_point;
 my_point.x = 10;
 my_point.y = 20;
 printf("X: %d Y: %d", my_point.x, my_point.y);
  
  return 0;
}

