import ctypes as ct

def Slice(elem_type):
    class Slice(ct.Structure):
        ELEMENT_TYPE = elem_type
        POINTER_TYPE = ct.POINTER(elem_type)

        _fields_ = [
            ('data', POINTER_TYPE),
            ('len', ct.c_size_t),
        ]
    
        def __len__(self):
            return int(self.len)

        def __iter__(self):
            count = len(self)
            for i in range(count):
                yield ptr[i]

    return Slice

def Owned(pointer_type):
    class Owned(ct.Structure):
        POINTER_TYPE = pointer_type
        DELETER_TYPE = ct.CFUNCTYPE(None, pointer_type)

        _fields_ = [
            ('data', POINTER_TYPE),
            ('deleter', DELETER_TYPE),
        ]

        def free(self):
            if self.deleter:
                self.deleter(self.data)
            self.data = None
            self.deleter = None

        def __del__(self):
            self.free()

    return Owned

def Closure(ret_type, *arg_types):
    class Closure(ct.Structure):
        FUNCTION_TYPE = ct.CFUNCTYPE(ret_type, ct.c_void_p, *arg_types)
        DELETER_TYPE = ct.CFUNCTYPE(None, ct.c_void_p)

        _fields_ = [
            ('call', FUNCTION_TYPE),
            ('state', ct.c_void_p),
            ('deleter', DELETER_TYPE),
        ]

        def __call__(self, *args):
            return self.call(self.state, *args)

        def free(self):
            if self.deleter:
                self.deleter(self.state)
            self.call = None
            self.state = None
            self.deleter = None

        def __del__(self):
            self.free()
