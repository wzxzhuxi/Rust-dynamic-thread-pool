// 引用比较和排序功能
use std::cmp::Ordering;
// 引用格式化功能，用于调试和打印
use std::fmt::{self, Debug};
// 引用哈希和散列功能
use std::hash::{Hash, Hasher};
// 引用从迭代器创建集合的功能
use std::iter::FromIterator;
// 引用标记功能，用于类型系统的高级特性
use std::marker::PhantomData;
// 引用指针功能，用于创建非空指针
use std::ptr::NonNull;

// 定义一个泛型链表结构体
pub struct LinkedList<T> {
    // 链表头指针，指向链表的第一个节点
    front: Link<T>,
    // 链表尾指针，指向链表的最后一个节点
    back: Link<T>,
    // 链表长度，表示链表中的元素个数
    len: usize,
    // 用于避免未使用类型参数T的警告
    _boo: PhantomData<T>,
}

// 定义一个链接节点类型为Option<NonNull<Node<T>>>，即节点的指针
type Link<T> = Option<NonNull<Node<T>>>;

// 定义链表节点结构体，包含前后指针和元素值
struct Node<T> {
    // 前一个节点的指针
    front: Link<T>,
    // 后一个节点的指针
    back: Link<T>,
    // 节点元素
    elem: T,
}

// 定义不可变迭代器结构体
pub struct Iter<'a, T> {
    // 当前前节点的指针
    front: Link<T>,
    // 当前后节点的指针
    back: Link<T>,
    // 剩余的元素个数
    len: usize,
    // 用于避免未使用生命周期和类型参数的警告
    _boo: PhantomData<&'a T>,
}

// 定义可变迭代器结构体
pub struct IterMut<'a, T> {
    // 当前前节点的指针
    front: Link<T>,
    // 当前后节点的指针
    back: Link<T>,
    // 剩余的元素个数
    len: usize,
    // 用于避免未使用生命周期和类型参数的警告
    _boo: PhantomData<&'a mut T>,
}

// 定义所有权迭代器结构体
pub struct IntoIter<T> {
    // 链表本身
    list: LinkedList<T>,
}

// 定义可变游标结构体
pub struct CursorMut<'a, T> {
    // 链表引用
    list: &'a mut LinkedList<T>,
    // 当前节点指针
    cur: Link<T>,
    // 当前索引位置
    index: Option<usize>,
}

impl<T> LinkedList<T> {
    // 创建一个新的空链表
    pub fn new() -> Self {
        Self {
            front: None, // 初始化链表头指针为空
            back: None, // 初始化链表尾指针为空
            len: 0, // 初始化链表长度为0
            _boo: PhantomData, // 初始化PhantomData
        }
    }

    // 在链表头部插入元素
    pub fn push_front(&mut self, elem: T) {
        unsafe {
            // 创建一个新的节点
            let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                front: None, // 新节点的前指针为空
                back: None, // 新节点的后指针为空
                elem, // 节点元素
            })));
            if let Some(old) = self.front {
                // 如果链表不为空，更新原头节点的前指针
                (*old.as_ptr()).front = Some(new);
                // 更新新节点的后指针
                (*new.as_ptr()).back = Some(old);
            } else {
                // 如果链表为空，更新链表尾指针
                self.back = Some(new);
            }
            // 更新链表头指针为新节点
            self.front = Some(new);
            // 增加链表长度
            self.len += 1;
        }
    }

    // 在链表尾部插入元素
    pub fn push_back(&mut self, elem: T) {
        unsafe {
            // 创建一个新的节点
            let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                back: None, // 新节点的后指针为空
                front: None, // 新节点的前指针为空
                elem, // 节点元素
            })));
            if let Some(old) = self.back {
                // 如果链表不为空，更新原尾节点的后指针
                (*old.as_ptr()).back = Some(new);
                // 更新新节点的前指针
                (*new.as_ptr()).front = Some(old);
            } else {
                // 如果链表为空，更新链表头指针
                self.front = Some(new);
            }
            // 更新链表尾指针为新节点
            self.back = Some(new);
            // 增加链表长度
            self.len += 1;
        }
    }

    // 移除并返回链表头部的元素
    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            // 如果链表头部存在节点，移除并返回节点元素
            self.front.map(|node| {
                let boxed_node = Box::from_raw(node.as_ptr()); // 获取头节点
                let result = boxed_node.elem; // 获取头节点元素

                self.front = boxed_node.back; // 更新链表头指针为头节点的后指针
                if let Some(new) = self.front {
                    // 如果链表不为空，更新新头节点的前指针为空
                    (*new.as_ptr()).front = None;
                } else {
                    // 如果链表为空，更新链表尾指针为空
                    self.back = None;
                }

                self.len -= 1; // 减少链表长度
                result // 返回节点元素
            })
        }
    }

    // 移除并返回链表尾部的元素
    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            // 如果链表尾部存在节点，移除并返回节点元素
            self.back.map(|node| {
                let boxed_node = Box::from_raw(node.as_ptr()); // 获取尾节点
                let result = boxed_node.elem; // 获取尾节点元素

                self.back = boxed_node.front; // 更新链表尾指针为尾节点的前指针
                if let Some(new) = self.back {
                    // 如果链表不为空，更新新尾节点的后指针为空
                    (*new.as_ptr()).back = None;
                } else {
                    // 如果链表为空，更新链表头指针为空
                    self.front = None;
                }

                self.len -= 1; // 减少链表长度
                result // 返回节点元素
            })
        }
    }

    // 返回链表头部的不可变引用
    pub fn front(&self) -> Option<&T> {
        unsafe { self.front.map(|node| &(*node.as_ptr()).elem) }
    }

    // 返回链表头部的可变引用
    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe { self.front.map(|node| &mut (*node.as_ptr()).elem) }
    }

    // 返回链表尾部的不可变引用
    pub fn back(&self) -> Option<&T> {
        unsafe { self.back.map(|node| &(*node.as_ptr()).elem) }
    }

    // 返回链表尾部的可变引用
    pub fn back_mut(&mut self) -> Option<&mut T> {
        unsafe { self.back.map(|node| &mut (*node.as_ptr()).elem) }
    }

    // 返回链表长度
    pub fn len(&self) -> usize {
        self.len
    }

    // 判断链表是否为空
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // 清空链表
    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    // 返回链表的不可变迭代器
    pub fn iter(&self) -> Iter<T> {
        Iter {
            front: self.front,
            back: self.back,
            len: self.len,
            _boo: PhantomData,
        }
    }

    // 返回链表的可变迭代器
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            front: self.front,
            back: self.back,
            len: self.len,
            _boo: PhantomData,
        }
    }

    // 返回链表的可变游标
    pub fn cursor_mut(&mut self) -> CursorMut<T> {
        CursorMut {
            list: self,
            cur: None,
            index: None,
        }
    }
}

// 实现链表的析构函数，确保在销毁前清空链表
impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

// 为链表实现默认特性，使其可以通过默认值创建
impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

// 为链表实现克隆特性，使其可以被克隆
impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        let mut new_list = Self::new();
        for item in self {
            new_list.push_back(item.clone());
        }
        new_list
    }
}

// 为链表实现Extend特性，使其可以通过迭代器扩展
impl<T> Extend<T> for LinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item);
        }
    }
}

// 为链表实现FromIterator特性，使其可以通过迭代器创建
impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

// 为链表实现Debug特性，使其可以被格式化输出
impl<T: Debug> Debug for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

// 为链表实现PartialEq特性，使其可以进行相等比较
impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

// 为链表实现Eq特性，使其可以进行严格相等比较
impl<T: Eq> Eq for LinkedList<T> {}

// 为链表实现PartialOrd特性，使其可以进行部分排序比较
impl<T: PartialOrd> PartialOrd for LinkedList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

// 为链表实现Ord特性，使其可以进行排序比较
impl<T: Ord> Ord for LinkedList<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

// 为链表实现Hash特性，使其可以进行哈希
impl<T: Hash> Hash for LinkedList<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for item in self {
            item.hash(state);
        }
    }
}

// 为不可变链表实现IntoIterator特性，使其可以转换为迭代器
impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// 为不可变迭代器实现Iterator特性，使其可以进行迭代
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.front.map(|node| unsafe {
                self.len -= 1;
                self.front = (*node.as_ptr()).back;
                &(*node.as_ptr()).elem
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

// 为不可变迭代器实现DoubleEndedIterator特性，使其可以进行双向迭代
impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|node| unsafe {
                self.len -= 1;
                self.back = (*node.as_ptr()).front;
                &(*node.as_ptr()).elem
            })
        } else {
            None
        }
    }
}

// 为不可变迭代器实现ExactSizeIterator特性，使其可以准确报告迭代器长度
impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

// 为可变链表实现IntoIterator特性，使其可以转换为迭代器
impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type IntoIter = IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// 为可变迭代器实现Iterator特性，使其可以进行迭代
impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.front.map(|node| unsafe {
                self.len -= 1;
                self.front = (*node.as_ptr()).back;
                &mut (*node.as_ptr()).elem
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

// 为可变迭代器实现DoubleEndedIterator特性，使其可以进行双向迭代
impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|node| unsafe {
                self.len -= 1;
                self.back = (*node.as_ptr()).front;
                &mut (*node.as_ptr()).elem
            })
        } else {
            None
        }
    }
}

// 为可变迭代器实现ExactSizeIterator特性，使其可以准确报告迭代器长度
impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

// 为链表实现IntoIterator特性，使其可以转换为迭代器
impl<T> IntoIterator for LinkedList<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}

// 为所有权迭代器实现Iterator特性，使其可以进行迭代
impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

// 为所有权迭代器实现DoubleEndedIterator特性，使其可以进行双向迭代
impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

// 为所有权迭代器实现ExactSizeIterator特性，使其可以准确报告迭代器长度
impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.list.len
    }
}

// 实现可变游标的各种操作方法
impl<'a, T> CursorMut<'a, T> {
    // 移动游标到下一个节点
    pub fn move_next(&mut self) {
        if let Some(node) = self.cur {
            unsafe {
                self.cur = (*node.as_ptr()).back;
                if self.cur.is_some() {
                    self.index = self.index.map(|i| i + 1);
                } else {
                    self.index = None;
                }
            }
        } else {
            self.cur = self.list.front;
            if self.cur.is_some() {
                self.index = Some(0);
            }
        }
    }

    // 移动游标到前一个节点
    pub fn move_prev(&mut self) {
        if let Some(node) = self.cur {
            unsafe {
                self.cur = (*node.as_ptr()).front;
                if self.cur.is_some() {
                    self.index = self.index.map(|i| i - 1);
                } else {
                    self.index = None;
                }
            }
        } else {
            self.cur = self.list.back;
            if self.cur.is_some() {
                self.index = Some(self.list.len - 1);
            }
        }
    }

    // 返回游标当前节点的可变引用
    pub fn current(&mut self) -> Option<&mut T> {
        unsafe { self.cur.map(|node| &mut (*node.as_ptr()).elem) }
    }

    // 返回游标当前节点的索引
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    // 查看游标下一个节点的元素
    pub fn peek_next(&self) -> Option<&T> {
        unsafe {
            self.cur.and_then(|node| (*node.as_ptr()).back.map(|next| &(*next.as_ptr()).elem))
        }
    }

    // 查看游标前一个节点的元素
    pub fn peek_prev(&self) -> Option<&T> {
        unsafe {
            self.cur.and_then(|node| (*node.as_ptr()).front.map(|prev| &(*prev.as_ptr()).elem))
        }
    }
}

// 为链表和迭代器实现Send和Sync特性，以便在多线程环境中使用
unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

unsafe impl<'a, T: Send> Send for Iter<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Iter<'a, T> {}

unsafe impl<'a, T: Send> Send for IterMut<'a, T> {}
unsafe impl<'a, T: Sync> Sync for IterMut<'a, T> {}
