// automatically generated by the FlatBuffers compiler, do not modify

package MCFS

import java.nio.*
import kotlin.math.sign
import com.google.flatbuffers.*

@Suppress("unused")
class WriteBody : Table() {

    fun __init(_i: Int, _bb: ByteBuffer)  {
        __reset(_i, _bb)
    }
    fun __assign(_i: Int, _bb: ByteBuffer) : WriteBody {
        __init(_i, _bb)
        return this
    }
    val float : Float?
        get() {
            val o = __offset(4)
            return if(o != 0) bb.getFloat(o + bb_pos) else null
        }
    val int : Int?
        get() {
            val o = __offset(6)
            return if(o != 0) bb.getInt(o + bb_pos) else null
        }
    val string : String?
        get() {
            val o = __offset(8)
            return if (o != 0) __string(o + bb_pos) else null
        }
    val stringAsByteBuffer : ByteBuffer get() = __vector_as_bytebuffer(8, 1)
    fun stringInByteBuffer(_bb: ByteBuffer) : ByteBuffer = __vector_in_bytebuffer(_bb, 8, 1)
    val vec : MCFS.Vec3? get() = vec(MCFS.Vec3())
    fun vec(obj: MCFS.Vec3) : MCFS.Vec3? {
        val o = __offset(10)
        return if (o != 0) {
            obj.__assign(o + bb_pos, bb)
        } else {
            null
        }
    }
    val block : MCFS.BlockPos? get() = block(MCFS.BlockPos())
    fun block(obj: MCFS.BlockPos) : MCFS.BlockPos? {
        val o = __offset(12)
        return if (o != 0) {
            obj.__assign(o + bb_pos, bb)
        } else {
            null
        }
    }
    companion object {
        fun validateVersion() = Constants.FLATBUFFERS_2_0_0()
        fun getRootAsWriteBody(_bb: ByteBuffer): WriteBody = getRootAsWriteBody(_bb, WriteBody())
        fun getRootAsWriteBody(_bb: ByteBuffer, obj: WriteBody): WriteBody {
            _bb.order(ByteOrder.LITTLE_ENDIAN)
            return (obj.__assign(_bb.getInt(_bb.position()) + _bb.position(), _bb))
        }
        fun startWriteBody(builder: FlatBufferBuilder) = builder.startTable(5)
        fun addFloat(builder: FlatBufferBuilder, float: Float) = builder.addFloat(0, float, 0.0)
        fun addInt(builder: FlatBufferBuilder, int: Int) = builder.addInt(1, int, 0)
        fun addString(builder: FlatBufferBuilder, string: Int) = builder.addOffset(2, string, 0)
        fun addVec(builder: FlatBufferBuilder, vec: Int) = builder.addStruct(3, vec, 0)
        fun addBlock(builder: FlatBufferBuilder, block: Int) = builder.addStruct(4, block, 0)
        fun endWriteBody(builder: FlatBufferBuilder) : Int {
            val o = builder.endTable()
            return o
        }
    }
}
