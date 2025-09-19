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
                yield self.data[i]

        def __getitem__(self, idx):
            return self.data[idx]

    return Slice

def Owned(pointer_type):
    class Owned(ct.Structure):
        POINTER_TYPE = pointer_type
        # Deleter always takes the whole pointer type (fat pointers by value).
        DELETER_TYPE = ct.CFUNCTYPE(None, pointer_type)

        _fields_ = [
            ('data', POINTER_TYPE),
            ('deleter', DELETER_TYPE),
        ]

        def _is_slice(self):
            return hasattr(self.POINTER_TYPE, 'ELEMENT_TYPE')

        def free(self):
            if getattr(self, '_freed', False):
                return
            try:
                if self.deleter:
                    # Call deleter. For slices, pass the underlying array pointer for robustness across FFI.
                    if self._is_slice():
                        self.deleter(self.data.data)
                    else:
                        self.deleter(self.data)
            finally:
                # Reset data to a benign default to avoid accidental reuse.
                if self._is_slice():
                    self.data = self.POINTER_TYPE()
                else:
                    self.data = None
                self._freed = True

        def __del__(self):
            try:
                self.free()
            except Exception:
                pass

        # If this Owned wraps a slice, allow direct iteration.
        def __iter__(self):
            if self._is_slice():
                return iter(self.data)
            raise TypeError('Owned value is not iterable')

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
