const array_size:usize = 5;
fn main() {
    println!("Hello world!");
    let mut ai : [i32; array_size] = [0;array_size];
    for i in 0..array_size {
        ai[i] = (array_size as i32) - (i as i32);
    }
    // ai.sort();
    let s = sort(ai);
    println!("{:?}", s);
}

fn sort(mut ar: [i32; array_size]) -> [i32;array_size] {
    let mut ra : [i32; array_size] = ar;
    for i in 0..array_size {
        let mi = min(&ar, i);
        let t = ar[i];
        ra[i] = ar[mi];
        ar[mi] = t;
    }
    return ra;
}

fn min(&ar: &[i32; array_size], si : usize) -> usize {
    let mut r : usize = si;
    for i in (si+1)..array_size {
        if ar[i] < ar[r] {
            r = i
        }
    }
    return r;
}

fn qsort(mut ar : [i32; array_size], pivot: usize) {

}

