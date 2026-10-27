define dso_local i32 @main(){
entry:
  

  %3 = alloca i32, align 4
  store i32 42, ptr %3, align 4
  %4 = load i32, ptr %3, align 4
  %5 = alloca ptr, align 8
  store ptr %3, ptr %5, align 8
  %6 = load ptr, ptr %5, align 8
  %7 = load i32, ptr %5, align 4
  ret i32 %7
  unreachable
}


