var __defProp = Object.defineProperty;
var __returnValue = (v) => v;
function __exportSetter(name, newValue) {
  this[name] = __returnValue.bind(null, newValue);
}
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, {
      get: all[name],
      enumerable: true,
      configurable: true,
      set: __exportSetter.bind(all, name)
    });
};

// action/src/vendor/malt_lib.js
var exports_malt_lib = {};
__export(exports_malt_lib, {
  Linter: () => Linter,
  default: () => __wbg_init,
  init: () => init,
  initSync: () => initSync,
  run_action: () => run_action
});

// action/src/index.mjs
import fs2 from "node:fs";
import { fileURLToPath } from "node:url";

// action/src/vendor/malt_lib.js
class Linter {
  static __wrap(ptr) {
    const obj = Object.create(Linter.prototype);
    obj.__wbg_ptr = ptr;
    LinterFinalization.register(obj, obj.__wbg_ptr, obj);
    return obj;
  }
  __destroy_into_raw() {
    const ptr = this.__wbg_ptr;
    this.__wbg_ptr = 0;
    LinterFinalization.unregister(this);
    return ptr;
  }
  free() {
    const ptr = this.__destroy_into_raw();
    wasm.__wbg_linter_free(ptr, 0);
  }
  check(content, path) {
    let deferred3_0;
    let deferred3_1;
    try {
      let ptr0, len0, ptr1, len1;
      try {
        ptr0 = passStringToWasm0(content, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        len0 = WASM_VECTOR_LEN;
        ptr1 = isLikeNone(path) ? 0 : passStringToWasm0(path, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        len1 = WASM_VECTOR_LEN;
      } catch (e) {
        if (ptr1) {
          wasm.__wbindgen_free_command_export(ptr1, len1 * 1, 1);
        }
        if (ptr0) {
          wasm.__wbindgen_free_command_export(ptr0, len0 * 1, 1);
        }
        throw e;
      }
      const ret = wasm.linter_check(this.__wbg_ptr, ptr0, len0, ptr1, len1);
      deferred3_0 = ret[0];
      deferred3_1 = ret[1];
      return getStringFromWasm0(ret[0], ret[1]);
    } finally {
      wasm.__wbindgen_free_command_export(deferred3_0, deferred3_1, 1);
    }
  }
  fix(content, path) {
    let deferred3_0;
    let deferred3_1;
    try {
      let ptr0, len0, ptr1, len1;
      try {
        ptr0 = passStringToWasm0(content, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        len0 = WASM_VECTOR_LEN;
        ptr1 = isLikeNone(path) ? 0 : passStringToWasm0(path, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
        len1 = WASM_VECTOR_LEN;
      } catch (e) {
        if (ptr1) {
          wasm.__wbindgen_free_command_export(ptr1, len1 * 1, 1);
        }
        if (ptr0) {
          wasm.__wbindgen_free_command_export(ptr0, len0 * 1, 1);
        }
        throw e;
      }
      const ret = wasm.linter_fix(this.__wbg_ptr, ptr0, len0, ptr1, len1);
      deferred3_0 = ret[0];
      deferred3_1 = ret[1];
      return getStringFromWasm0(ret[0], ret[1]);
    } finally {
      wasm.__wbindgen_free_command_export(deferred3_0, deferred3_1, 1);
    }
  }
  static from_toml(toml) {
    const ptr0 = passStringToWasm0(toml, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.linter_from_toml(ptr0, len0);
    if (ret[2]) {
      throw takeFromExternrefTable0(ret[1]);
    }
    return Linter.__wrap(ret[0]);
  }
  static from_toml_with(toml, overrides) {
    let ptr0, len0, ptr1, len1;
    try {
      ptr0 = passStringToWasm0(toml, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
      len0 = WASM_VECTOR_LEN;
      ptr1 = passStringToWasm0(overrides, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
      len1 = WASM_VECTOR_LEN;
    } catch (e) {
      if (ptr1) {
        wasm.__wbindgen_free_command_export(ptr1, len1 * 1, 1);
      }
      if (ptr0) {
        wasm.__wbindgen_free_command_export(ptr0, len0 * 1, 1);
      }
      throw e;
    }
    const ret = wasm.linter_from_toml_with(ptr0, len0, ptr1, len1);
    if (ret[2]) {
      throw takeFromExternrefTable0(ret[1]);
    }
    return Linter.__wrap(ret[0]);
  }
  static get_available_rules() {
    let deferred1_0;
    let deferred1_1;
    try {
      const ret = wasm.linter_get_available_rules();
      deferred1_0 = ret[0];
      deferred1_1 = ret[1];
      return getStringFromWasm0(ret[0], ret[1]);
    } finally {
      wasm.__wbindgen_free_command_export(deferred1_0, deferred1_1, 1);
    }
  }
  get_config_warnings() {
    let deferred1_0;
    let deferred1_1;
    try {
      const ret = wasm.linter_get_config_warnings(this.__wbg_ptr);
      deferred1_0 = ret[0];
      deferred1_1 = ret[1];
      return getStringFromWasm0(ret[0], ret[1]);
    } finally {
      wasm.__wbindgen_free_command_export(deferred1_0, deferred1_1, 1);
    }
  }
  static get_version() {
    let deferred1_0;
    let deferred1_1;
    try {
      const ret = wasm.linter_get_version();
      deferred1_0 = ret[0];
      deferred1_1 = ret[1];
      return getStringFromWasm0(ret[0], ret[1]);
    } finally {
      wasm.__wbindgen_free_command_export(deferred1_0, deferred1_1, 1);
    }
  }
  constructor(options) {
    const ret = wasm.linter_new(options);
    if (ret[2]) {
      throw takeFromExternrefTable0(ret[1]);
    }
    this.__wbg_ptr = ret[0];
    LinterFinalization.register(this, this.__wbg_ptr, this);
    return this;
  }
}
if (Symbol.dispose)
  Linter.prototype[Symbol.dispose] = Linter.prototype.free;
function init() {
  wasm.init();
}
function run_action(inputs, stat, list, read) {
  let deferred3_0;
  let deferred3_1;
  try {
    const ptr0 = passStringToWasm0(inputs, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.run_action(ptr0, len0, stat, list, read);
    var ptr2 = ret[0];
    var len2 = ret[1];
    if (ret[3]) {
      ptr2 = 0;
      len2 = 0;
      throw takeFromExternrefTable0(ret[2]);
    }
    deferred3_0 = ptr2;
    deferred3_1 = len2;
    return getStringFromWasm0(ptr2, len2);
  } finally {
    wasm.__wbindgen_free_command_export(deferred3_0, deferred3_1, 1);
  }
}
function __wbg_get_imports() {
  const import0 = {
    __proto__: null,
    __wbg_Error_208446807d0d1e30: function(arg0, arg1) {
      const ret = Error(getStringFromWasm0(arg0, arg1));
      return ret;
    },
    __wbg_String_8564e559799eccda: function(arg0, arg1) {
      const ret = String(arg1);
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
    },
    __wbg___wbindgen_bigint_get_as_i64_11da4b52cd0aedef: function(arg0, arg1) {
      const v = arg1;
      const ret = typeof v === "bigint" ? v : undefined;
      getDataViewMemory0().setBigInt64((arg0 >>> 0) + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, !isLikeNone(ret), true);
    },
    __wbg___wbindgen_boolean_get_4f1baf9a00be3c74: function(arg0) {
      const v = arg0;
      const ret = typeof v === "boolean" ? v : undefined;
      return isLikeNone(ret) ? 16777215 : ret ? 1 : 0;
    },
    __wbg___wbindgen_debug_string_f249a2a76caa9a55: function(arg0, arg1) {
      const ret = debugString(arg1);
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
    },
    __wbg___wbindgen_in_3f58a9d3b3a7eed2: function(arg0, arg1) {
      const ret = arg0 in arg1;
      return ret;
    },
    __wbg___wbindgen_is_bigint_5fa10641e039d3ef: function(arg0) {
      const ret = typeof arg0 === "bigint";
      return ret;
    },
    __wbg___wbindgen_is_function_741ff2bb612836f5: function(arg0) {
      const ret = typeof arg0 === "function";
      return ret;
    },
    __wbg___wbindgen_is_null_c22311b1cc228e3a: function(arg0) {
      const ret = arg0 === null;
      return ret;
    },
    __wbg___wbindgen_is_object_8b968c3ab8e96595: function(arg0) {
      const val = arg0;
      const ret = typeof val === "object" && val !== null;
      return ret;
    },
    __wbg___wbindgen_is_undefined_be5b34249ff1fc15: function(arg0) {
      const ret = arg0 === undefined;
      return ret;
    },
    __wbg___wbindgen_jsval_eq_38d7bfbe4cc84039: function(arg0, arg1) {
      const ret = arg0 === arg1;
      return ret;
    },
    __wbg___wbindgen_jsval_loose_eq_b2f214e2e4b30e36: function(arg0, arg1) {
      const ret = arg0 == arg1;
      return ret;
    },
    __wbg___wbindgen_number_get_934b640d0889af78: function(arg0, arg1) {
      const obj = arg1;
      const ret = typeof obj === "number" ? obj : undefined;
      getDataViewMemory0().setFloat64((arg0 >>> 0) + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, !isLikeNone(ret), true);
    },
    __wbg___wbindgen_string_get_7ce8eee7fd327545: function(arg0, arg1) {
      const obj = arg1;
      const ret = typeof obj === "string" ? obj : undefined;
      var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
      var len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
    },
    __wbg___wbindgen_throw_4c88d8bf264fd984: function(arg0, arg1) {
      throw new Error(getStringFromWasm0(arg0, arg1));
    },
    __wbg_call_40b008ba32f73991: function() {
      return handleError(function(arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
      }, arguments);
    },
    __wbg_call_a410eb4b59ed5965: function() {
      return handleError(function(arg0, arg1, arg2) {
        const ret = arg0.call(arg1, arg2);
        return ret;
      }, arguments);
    },
    __wbg_done_57e08d0be6e30c02: function(arg0) {
      const ret = arg0.done;
      return ret;
    },
    __wbg_entries_7bb1459bd9b5c2b9: function(arg0) {
      const ret = Object.entries(arg0);
      return ret;
    },
    __wbg_error_757e9472f8410341: function(arg0, arg1) {
      let deferred0_0;
      let deferred0_1;
      try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.error(getStringFromWasm0(arg0, arg1));
      } finally {
        wasm.__wbindgen_free_command_export(deferred0_0, deferred0_1, 1);
      }
    },
    __wbg_from_eac91875c8a5ffa4: function(arg0) {
      const ret = Array.from(arg0);
      return ret;
    },
    __wbg_get_635eea311fa1a291: function() {
      return handleError(function(arg0, arg1) {
        const ret = Reflect.get(arg0, arg1);
        return ret;
      }, arguments);
    },
    __wbg_get_d7176cfb76d71114: function(arg0, arg1) {
      const ret = arg0[arg1 >>> 0];
      return ret;
    },
    __wbg_get_unchecked_e6325ad374eff861: function(arg0, arg1) {
      const ret = arg0[arg1 >>> 0];
      return ret;
    },
    __wbg_instanceof_ArrayBuffer_12cdaa4161423e6b: function(arg0) {
      let result;
      try {
        result = arg0 instanceof ArrayBuffer;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    },
    __wbg_instanceof_Map_0b4fc8b62e73f163: function(arg0) {
      let result;
      try {
        result = arg0 instanceof Map;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    },
    __wbg_instanceof_Uint8Array_d7aeb0dcb3108fea: function(arg0) {
      let result;
      try {
        result = arg0 instanceof Uint8Array;
      } catch (_) {
        result = false;
      }
      const ret = result;
      return ret;
    },
    __wbg_isArray_bcf1376d0d732d08: function(arg0) {
      const ret = Array.isArray(arg0);
      return ret;
    },
    __wbg_isSafeInteger_1a35ba4d31e71d38: function(arg0) {
      const ret = Number.isSafeInteger(arg0);
      return ret;
    },
    __wbg_iterator_e4c5e8f68ada7b58: function() {
      const ret = Symbol.iterator;
      return ret;
    },
    __wbg_length_186163212ffe79c3: function(arg0) {
      const ret = arg0.length;
      return ret;
    },
    __wbg_length_b03a0fabb109b1e9: function(arg0) {
      const ret = arg0.length;
      return ret;
    },
    __wbg_new_13de428334042d26: function(arg0) {
      const ret = new Uint8Array(arg0);
      return ret;
    },
    __wbg_new_bb74001155cb8ca8: function() {
      const ret = new Error;
      return ret;
    },
    __wbg_next_178c1651c11432b3: function(arg0) {
      const ret = arg0.next;
      return ret;
    },
    __wbg_next_af5ac6a08bb0a717: function() {
      return handleError(function(arg0) {
        const ret = arg0.next();
        return ret;
      }, arguments);
    },
    __wbg_prototypesetcall_13162965d1fe7aa3: function(arg0, arg1, arg2) {
      Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
    },
    __wbg_stack_575c4e0918581b9c: function(arg0, arg1) {
      const ret = arg1.stack;
      const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc_command_export, wasm.__wbindgen_realloc_command_export);
      const len1 = WASM_VECTOR_LEN;
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
      getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
    },
    __wbg_value_e0f328965e184931: function(arg0) {
      const ret = arg0.value;
      return ret;
    },
    __wbindgen_generic_0000000000000001: function(arg0) {
      const ret = arg0;
      return ret;
    },
    __wbindgen_generic_0000000000000002: function(arg0, arg1) {
      const ret = getStringFromWasm0(arg0, arg1);
      return ret;
    },
    __wbindgen_generic_0000000000000003: function(arg0) {
      const ret = BigInt.asUintN(64, arg0);
      return ret;
    },
    __wbindgen_init_externref_table: function() {
      const table = wasm.__wbindgen_externrefs;
      table.set(0, undefined);
      table.set(1024, undefined);
      table.set(1025, null);
      table.set(1026, true);
      table.set(1027, false);
    }
  };
  return {
    __proto__: null,
    "./malt_lib_bg.js": import0
  };
}
var LinterFinalization = typeof FinalizationRegistry === "undefined" ? { register: () => {}, unregister: () => {} } : new FinalizationRegistry((ptr) => wasm.__wbg_linter_free(ptr, 1));
function addToExternrefTable0(obj) {
  const idx = wasm.__externref_table_alloc_command_export();
  wasm.__wbindgen_externrefs.set(idx, obj);
  return idx;
}
function debugString(val) {
  switch (typeof val) {
    case "number":
    case "boolean":
    case "undefined":
      return `${val}`;
    case "string":
      return JSON.stringify(val);
    case "bigint":
      return `${val}n`;
    case "symbol":
      return val.description !== undefined ? `Symbol(${val.description})` : "Symbol";
    case "function":
      return typeof val.name === "string" && val.name !== "" ? `Function(${val.name})` : "Function";
    case "object": {
      if (val === null) {
        return "null";
      }
      if (Array.isArray(val)) {
        return "[" + Array.from(val, debugString).join(", ") + "]";
      }
      if (val instanceof Error) {
        return `${val.name}: ${val.message}
${val.stack}`;
      }
      const toStringOutput = toString.call(val);
      const className = /\[object ([^\]]+)\]/.exec(toStringOutput)?.[1];
      if (className === undefined) {
        return toStringOutput;
      }
      if (className === "Object") {
        try {
          return "Object(" + JSON.stringify(val) + ")";
        } catch (_) {
          return "Object";
        }
      }
      return className;
    }
  }
}
function getArrayU8FromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}
var cachedDataViewMemory0 = null;
function getDataViewMemory0() {
  if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer) {
    cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
  }
  return cachedDataViewMemory0;
}
function getStringFromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return decodeText(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}
var cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
  if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
    cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8ArrayMemory0;
}
function handleError(f, args) {
  try {
    return f.apply(this, args);
  } catch (e) {
    const idx = addToExternrefTable0(e);
    wasm.__wbindgen_exn_store_command_export(idx);
  }
}
function isLikeNone(x) {
  return x === undefined || x === null;
}
function passStringToWasm0(arg, malloc, realloc) {
  if (realloc === undefined) {
    const buf = cachedTextEncoder.encode(arg);
    const ptr = malloc(buf.length, 1) >>> 0;
    getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
    WASM_VECTOR_LEN = buf.length;
    return ptr;
  }
  let len = arg.length;
  let ptr = malloc(len, 1) >>> 0;
  const mem = getUint8ArrayMemory0();
  let offset = 0;
  for (;offset < len; offset++) {
    const code = arg.charCodeAt(offset);
    if (code > 127)
      break;
    mem[ptr + offset] = code;
  }
  if (offset !== len) {
    if (offset !== 0) {
      arg = arg.slice(offset);
    }
    ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
    const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
    const ret = cachedTextEncoder.encodeInto(arg, view);
    offset += ret.written;
    ptr = realloc(ptr, len, offset, 1) >>> 0;
  }
  WASM_VECTOR_LEN = offset;
  return ptr;
}
function takeFromExternrefTable0(idx) {
  const value = wasm.__wbindgen_externrefs.get(idx);
  wasm.__externref_table_dealloc_command_export(idx);
  return value;
}
var cachedTextDecoder = new TextDecoder("utf-8", { ignoreBOM: true });
cachedTextDecoder.decode();
var MAX_SAFARI_DECODE_BYTES = 2146435072;
var numBytesDecoded = 0;
function decodeText(bytes) {
  numBytesDecoded += bytes.length;
  if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
    cachedTextDecoder = new TextDecoder("utf-8", { ignoreBOM: true });
    cachedTextDecoder.decode();
    numBytesDecoded = bytes.length;
  }
  return cachedTextDecoder.decode(bytes);
}
var cachedTextEncoder = new TextEncoder;
if (!("encodeInto" in cachedTextEncoder)) {
  cachedTextEncoder.encodeInto = function(arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
      read: arg.length,
      written: buf.length
    };
  };
}
var WASM_VECTOR_LEN = 0;
var wasmModule;
var wasmInstance;
var wasm;
function __wbg_finalize_init(instance, module) {
  wasmInstance = instance;
  wasm = instance.exports;
  wasmModule = module;
  cachedDataViewMemory0 = null;
  cachedUint8ArrayMemory0 = null;
  wasm.__wbindgen_start();
  return wasm;
}
async function __wbg_load(module, imports) {
  if (typeof Response === "function" && module instanceof Response) {
    if (!module.ok) {
      throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
    }
    if (typeof WebAssembly.instantiateStreaming === "function") {
      try {
        return await WebAssembly.instantiateStreaming(module, imports);
      } catch (e) {
        const validResponse = expectedResponseType(module.type);
        if (validResponse && module.headers.get("Content-Type") !== "application/wasm") {
          console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
        } else {
          throw e;
        }
      }
    }
    const bytes = await module.arrayBuffer();
    return await WebAssembly.instantiate(bytes, imports);
  } else {
    const instance = await WebAssembly.instantiate(module, imports);
    if (instance instanceof WebAssembly.Instance) {
      return { instance, module };
    } else {
      return instance;
    }
  }
  function expectedResponseType(type) {
    switch (type) {
      case "basic":
      case "cors":
      case "default":
        return true;
    }
    return false;
  }
}
function initSync(module) {
  if (wasm !== undefined)
    return wasm;
  if (module !== undefined) {
    if (Object.getPrototypeOf(module) === Object.prototype) {
      ({ module } = module);
    } else {
      console.warn("using deprecated parameters for `initSync()`; pass a single object instead");
    }
  }
  const imports = __wbg_get_imports();
  if (!(module instanceof WebAssembly.Module)) {
    module = new WebAssembly.Module(module);
  }
  const instance = new WebAssembly.Instance(module, imports);
  return __wbg_finalize_init(instance, module);
}
var __wbg_init_promise;
async function __wbg_init(module_or_path) {
  if (wasm !== undefined)
    return wasm;
  if (__wbg_init_promise === undefined) {
    __wbg_init_promise = __wbg_init_once(module_or_path);
    __wbg_init_promise.catch(() => {
      __wbg_init_promise = undefined;
    });
  }
  return __wbg_init_promise;
}
async function __wbg_init_once(module_or_path) {
  if (module_or_path !== undefined) {
    if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
      ({ module_or_path } = module_or_path);
    } else {
      console.warn("using deprecated parameters for the initialization function; pass a single object instead");
    }
  }
  if (module_or_path === undefined) {
    module_or_path = new URL("malt_lib_bg.wasm", import.meta.url);
  }
  const imports = __wbg_get_imports();
  if (typeof module_or_path === "string" || typeof Request === "function" && module_or_path instanceof Request || typeof URL === "function" && module_or_path instanceof URL) {
    module_or_path = fetch(module_or_path);
  }
  const { instance, module } = await __wbg_load(await module_or_path, imports);
  return __wbg_finalize_init(instance, module);
}

// action/src/shell.mjs
import fs from "node:fs";
import path from "node:path";
var INPUTS = ["paths", "preset", "level", "fail-on", "annotations", "summary", "output-json"];
function kindOf(entry) {
  if (entry.isDirectory())
    return "dir";
  if (entry.isFile())
    return "file";
  return "other";
}
function runShell({ wasm, env = process.env, cwd = process.cwd(), out = process.stdout }) {
  const inputs = { workspace: env.GITHUB_WORKSPACE ?? cwd };
  for (const name of INPUTS)
    inputs[name] = env[`INPUT_${name.toUpperCase()}`] ?? "";
  const resolve = (relative) => path.join(cwd, relative);
  const stat = (relative) => {
    try {
      return kindOf(fs.statSync(resolve(relative)));
    } catch {
      return null;
    }
  };
  const list = (dir) => fs.readdirSync(resolve(dir), { withFileTypes: true }).map((entry) => [entry.name, kindOf(entry)]);
  const read = (relative) => {
    try {
      return fs.readFileSync(resolve(relative));
    } catch {
      return null;
    }
  };
  const outcome = JSON.parse(wasm.run_action(JSON.stringify(inputs), stat, list, read));
  out.write(outcome.stdout);
  if (outcome.summary_markdown && env.GITHUB_STEP_SUMMARY) {
    fs.appendFileSync(env.GITHUB_STEP_SUMMARY, outcome.summary_markdown);
  }
  const results = inputs["output-json"].trim();
  if (outcome.results_json !== null && results) {
    const file = path.resolve(cwd, results);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, outcome.results_json);
  }
  if (env.GITHUB_OUTPUT) {
    fs.appendFileSync(env.GITHUB_OUTPUT, `findings=${outcome.findings}
errors=${outcome.errors}
warnings=${outcome.warnings}
` + `results-file=${results ? path.resolve(cwd, results) : ""}
`);
  }
  return outcome;
}

// action/src/index.mjs
try {
  const bytes = fs2.readFileSync(fileURLToPath(new URL("./malt_lib_bg.wasm", import.meta.url)));
  await __wbg_init({ module_or_path: bytes });
  if (runShell({ wasm: exports_malt_lib }).failed)
    process.exitCode = 1;
} catch (error) {
  const message = String(error?.message ?? error).replace(/%/g, "%25").replace(/\r?\n/g, "%0A");
  process.stdout.write(`::error title=malt::${message}
`);
  process.exitCode = 1;
}
