// automatically generated by the FlatBuffers compiler, do not modify

package MCFS.Common

import java.nio.*
import kotlin.math.sign
import com.google.flatbuffers.*

@Suppress("unused")
@ExperimentalUnsignedTypes
class Vec3 : Struct() {

    fun __init(_i: Int, _bb: ByteBuffer)  {
        __reset(_i, _bb)
    }
    fun __assign(_i: Int, _bb: ByteBuffer) : Vec3 {
        __init(_i, _bb)
        return this
    }
    val x : Double get() = bb.getDouble(bb_pos + 0)
    val y : Double get() = bb.getDouble(bb_pos + 8)
    val z : Double get() = bb.getDouble(bb_pos + 16)
    companion object {
        fun createVec3(builder: FlatBufferBuilder, x: Double, y: Double, z: Double) : Int {
            builder.prep(8, 24)
            builder.putDouble(z)
            builder.putDouble(y)
            builder.putDouble(x)
            return builder.offset()
        }
    }
}