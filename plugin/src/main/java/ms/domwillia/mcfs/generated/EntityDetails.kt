// automatically generated by the FlatBuffers compiler, do not modify

package MCFS

import java.nio.*
import kotlin.math.sign
import com.google.flatbuffers.*

@Suppress("unused")
@ExperimentalUnsignedTypes
class EntityDetails : Struct() {

    fun __init(_i: Int, _bb: ByteBuffer)  {
        __reset(_i, _bb)
    }
    fun __assign(_i: Int, _bb: ByteBuffer) : EntityDetails {
        __init(_i, _bb)
        return this
    }
    val id : Int get() = bb.getInt(bb_pos + 0)
    val living : Boolean get() = 0.toByte() != bb.get(bb_pos + 4)
    companion object {
        fun createEntityDetails(builder: FlatBufferBuilder, id: Int, living: Boolean) : Int {
            builder.prep(4, 8)
            builder.pad(3)
            builder.putBoolean(living)
            builder.putInt(id)
            return builder.offset()
        }
    }
}
