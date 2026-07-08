typedef struct {
    int x;
} StructAnon;

StructAnon v23 = {1};

typedef struct Point {
    int x;
    int y;
} Point;

Point v24 = {1,2};

typedef struct Node Node;

struct Node {
    int value;
    Node *next;
};

Node v25 = {10, 0};
