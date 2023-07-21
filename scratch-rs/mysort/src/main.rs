const ARRAY_SIZE:usize = 50000;
fn main() {
    println!("Hello world!");
    let mut ai : [i32; ARRAY_SIZE] = [0;ARRAY_SIZE];
    for i in 0..ARRAY_SIZE {
        ai[i] = (ARRAY_SIZE as i32) - (i as i32);
    }
    // ai.sort();
    // let s = sort(ai);
    // println!("{:?}", s);
    qsort(&mut ai, 0, ARRAY_SIZE-1);
    //ai.sort();
    //println!("{:?}", ai);
}

fn sort(mut ar: [i32; ARRAY_SIZE]) -> [i32;ARRAY_SIZE] {
    let mut ra : [i32; ARRAY_SIZE] = ar;
    for i in 0..ARRAY_SIZE {
        let mi = min(&ar, i);
        let t = ar[i];
        ra[i] = ar[mi];
        ar[mi] = t;
    }
    return ra;
}

fn min(&ar: &[i32; ARRAY_SIZE], si : usize) -> usize {
    let mut r : usize = si;
    for i in (si+1)..ARRAY_SIZE {
        if ar[i] < ar[r] {
            r = i
        }
    }
    return r;
}

fn qsort(arr : &mut [i32; ARRAY_SIZE], lo: usize, ro: usize) {
    if ro == lo {
        return;
    }
    if ro -lo == 1 {
        if arr[lo] > arr[ro] {
            arr.swap(lo, ro);
        }
        return
    }
    let mut pivot: usize = (lo + ro) / 2;
    let pivot_value = arr[pivot];
    let mut i = pivot - 1;
    loop {
        if arr[i] > pivot_value {
            let t = arr[i];
            for j in i..pivot {
                arr[j] = arr[j+1];
            }
            arr[pivot] = t;
            pivot = pivot - 1;
        }
        if i == lo { break }
        i = i - 1;
    }

    i = pivot + 1;
    loop {
        if arr[i] < pivot_value {
            let t = arr[i];

            let mut j = i;
            loop {
                arr[j] = arr[j-1];
                j = j - 1;
                if j == pivot { break }
            }
            arr[pivot] = t;
            pivot = pivot + 1;
        }
        if i == ro { break }
        i = i + 1;
    }
    qsort(arr, lo, pivot - 1);
    qsort(arr, pivot + 1, ro);
}

