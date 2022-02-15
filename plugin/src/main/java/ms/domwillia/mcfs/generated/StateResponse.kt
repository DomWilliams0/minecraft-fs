// automatically generated by the FlatBuffers compiler, do not modify

package MCFS

import java.nio.*
import kotlin.math.sign
import com.google.flatbuffers.*

@Suppress("unused")
@ExperimentalUnsignedTypes
class StateResponse : Table() {

    fun __init(_i: Int, _bb: ByteBuffer)  {
        __reset(_i, _bb)
    }
    fun __assign(_i: Int, _bb: ByteBuffer) : StateResponse {
        __init(_i, _bb)
        return this
    }
    val playerEntityId : Int?
        get() {
            val o = __offset(4)
            return if(o != 0) bb.getInt(o + bb_pos) else null
        }
    val playerWorld : UByte?
        get() {
            val o = __offset(6)
            return if(o != 0) bb.get(o + bb_pos).toUByte() else null
        }
    fun entityIds(j: Int) : Int {
        val o = __offset(8)
        return if (o != 0) {
            bb.getInt(__vector(o) + j * 4)
        } else {
            0
        }
    }
    val entityIdsLength : Int
        get() {
            val o = __offset(8); return if (o != 0) __vector_len(o) else 0
        }
    val entityIdsAsByteBuffer : ByteBuffer get() = __vector_as_bytebuffer(8, 4)
    fun entityIdsInByteBuffer(_bb: ByteBuffer) : ByteBuffer = __vector_in_bytebuffer(_bb, 8, 4)
    val block : MCFS.BlockDetails? get() = block(MCFS.BlockDetails())
    fun block(obj: MCFS.BlockDetails) : MCFS.BlockDetails? {
        val o = __offset(10)
        return if (o != 0) {
            obj.__assign(o + bb_pos, bb)
        } else {
            null
        }
    }
    companion object {
        fun validateVersion() = Constants.FLATBUFFERS_2_0_0()
        fun getRootAsStateResponse(_bb: ByteBuffer): StateResponse = getRootAsStateResponse(_bb, StateResponse())
        fun getRootAsStateResponse(_bb: ByteBuffer, obj: StateResponse): StateResponse {
            _bb.order(ByteOrder.LITTLE_ENDIAN)
            return (obj.__assign(_bb.getInt(_bb.position()) + _bb.position(), _bb))
        }
        fun startStateResponse(builder: FlatBufferBuilder) = builder.startTable(4)
        fun addPlayerEntityId(builder: FlatBufferBuilder, playerEntityId: Int) = builder.addInt(0, playerEntityId, 0)
        fun addPlayerWorld(builder: FlatBufferBuilder, playerWorld: UByte) = builder.addByte(1, playerWorld.toByte(), 0)
        fun addEntityIds(builder: FlatBufferBuilder, entityIds: Int) = builder.addOffset(2, entityIds, 0)
        fun createEntityIdsVector(builder: FlatBufferBuilder, data: IntArray) : Int {
            builder.startVector(4, data.size, 4)
            for (i in data.size - 1 downTo 0) {
                builder.addInt(data[i])
            }
            return builder.endVector()
        }
        fun startEntityIdsVector(builder: FlatBufferBuilder, numElems: Int) = builder.startVector(4, numElems, 4)
        fun addBlock(builder: FlatBufferBuilder, block: Int) = builder.addStruct(3, block, 0)
        fun endStateResponse(builder: FlatBufferBuilder) : Int {
            val o = builder.endTable()
            return o
        }
    }
}
