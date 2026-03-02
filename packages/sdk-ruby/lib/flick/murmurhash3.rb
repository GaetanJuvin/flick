# frozen_string_literal: true

module Flick
  module MurmurHash3
    MASK32 = 0xFFFFFFFF
    C1 = 0xcc9e2d51
    C2 = 0x1b873593

    module_function

    # MurmurHash3 (32-bit) — deterministic hash matching TS and Kotlin SDKs.
    # Ruby's arbitrary-precision integers require masking on every operation.
    def hash(key, seed = 0)
      h1 = seed & MASK32
      len = key.bytesize
      bytes = key.bytes

      i = 0
      while i + 4 <= len
        k1 = (bytes[i] & 0xff) |
             ((bytes[i + 1] & 0xff) << 8) |
             ((bytes[i + 2] & 0xff) << 16) |
             ((bytes[i + 3] & 0xff) << 24)

        k1 = (k1 * C1) & MASK32
        k1 = ((k1 << 15) | (k1 >> 17)) & MASK32
        k1 = (k1 * C2) & MASK32

        h1 ^= k1
        h1 = ((h1 << 13) | (h1 >> 19)) & MASK32
        h1 = (h1 * 5 + 0xe6546b64) & MASK32

        i += 4
      end

      k1 = 0
      remaining = len & 3
      if remaining >= 3
        k1 ^= (bytes[i + 2] & 0xff) << 16
      end
      if remaining >= 2
        k1 ^= (bytes[i + 1] & 0xff) << 8
      end
      if remaining >= 1
        k1 ^= (bytes[i] & 0xff)
        k1 = (k1 * C1) & MASK32
        k1 = ((k1 << 15) | (k1 >> 17)) & MASK32
        k1 = (k1 * C2) & MASK32
        h1 ^= k1
      end

      h1 ^= len
      h1 ^= (h1 >> 16)
      h1 = (h1 * 0x85ebca6b) & MASK32
      h1 ^= (h1 >> 13)
      h1 = (h1 * 0xc2b2ae35) & MASK32
      h1 ^= (h1 >> 16)

      h1
    end
  end
end
