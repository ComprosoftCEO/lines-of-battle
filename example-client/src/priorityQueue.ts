// Implementation is modified from:
// https://stackoverflow.com/questions/42919469/efficient-way-to-implement-priority-queue-in-javascript
const TOP = 0;
const parent = (i) => ((i + 1) >>> 1) - 1;
const left = (i) => (i << 1) + 1;
const right = (i) => (i + 1) << 1;

export class PriorityQueue<T> {
  private _heap: T[];
  private _comparator: (a: T, b: T) => boolean;

  constructor(comparator = (a: T, b: T): boolean => (a as any) > (b as any)) {
    this._heap = [];
    this._comparator = comparator;
  }

  size(): number {
    return this._heap.length;
  }

  isEmpty(): boolean {
    return this.size() == 0;
  }

  peek(): T {
    return this._heap[TOP];
  }

  push(...values: T[]): number {
    values.forEach((value) => {
      this._heap.push(value);
      this._siftUp();
    });
    return this.size();
  }

  pop(): T {
    const poppedValue = this.peek();
    const bottom = this.size() - 1;
    if (bottom > TOP) {
      this._swap(TOP, bottom);
    }
    this._heap.pop();
    this._siftDown();
    return poppedValue;
  }

  replace(value: T): T {
    const replacedValue = this.peek();
    this._heap[TOP] = value;
    this._siftDown();
    return replacedValue;
  }

  private _greater(i, j) {
    return this._comparator(this._heap[i], this._heap[j]);
  }

  private _swap(i, j) {
    [this._heap[i], this._heap[j]] = [this._heap[j], this._heap[i]];
  }

  private _siftUp() {
    let node = this.size() - 1;
    while (node > TOP && this._greater(node, parent(node))) {
      this._swap(node, parent(node));
      node = parent(node);
    }
  }

  private _siftDown() {
    let node = TOP;
    while (
      (left(node) < this.size() && this._greater(left(node), node)) ||
      (right(node) < this.size() && this._greater(right(node), node))
    ) {
      const maxChild =
        right(node) < this.size() && this._greater(right(node), left(node))
          ? right(node)
          : left(node);
      this._swap(node, maxChild);
      node = maxChild;
    }
  }
}
