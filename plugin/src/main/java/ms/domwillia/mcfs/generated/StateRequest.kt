// automatically generated by the FlatBuffers compiler, do not modify

package MCFS

import java.nio.*
import kotlin.math.sign
import com.google.flatbuffers.*

@Suppress("unused")
class StateRequest : Table() {

    fun __init(_i: Int, _bb: ByteBuffer)  {
        __reset(_i, _bb)
    }
    fun __assign(_i: Int, _bb: ByteBuffer) : StateRequest {
        __init(_i, _bb)
        return this
    }
    val entitiesById : Boolean
        get() {
            val o = __offset(4)
            return if(o != 0) 0.toByte() != bb.get(o + bb_pos) else false
        }
    val targetWorld : UByte?
        get() {
            val o = __offset(6)
            return if(o != 0) bb.get(o + bb_pos).toUByte() else null
        }
    val targetBlock : MCFS.BlockPos? get() = targetBlock(MCFS.BlockPos())
    fun targetBlock(obj: MCFS.BlockPos) : MCFS.BlockPos? {
        val o = __offset(8)
        return if (o != 0) {
            obj.__assign(o + bb_pos, bb)
        } else {
            null
        }
    }
    companion object {
        fun validateVersion() = Constants.FLATBUFFERS_2_0_0()
        fun getRootAsStateRequest(_bb: ByteBuffer): StateRequest = getRootAsStateRequest(_bb, StateRequest())
        fun getRootAsStateRequest(_bb: ByteBuffer, obj: StateRequest): StateRequest {
            _bb.order(ByteOrder.LITTLE_ENDIAN)
            return (obj.__assign(_bb.getInt(_bb.position()) + _bb.position(), _bb))
        }
        fun startStateRequest(builder: FlatBufferBuilder) = builder.startTable(3)
        fun addEntitiesById(builder: FlatBufferBuilder, entitiesById: Boolean) = builder.addBoolean(0, entitiesById, false)
        fun addTargetWorld(builder: FlatBufferBuilder, targetWorld: UByte) = builder.addByte(1, targetWorld.toByte(), 0)
        fun addTargetBlock(builder: FlatBufferBuilder, targetBlock: Int) = builder.addStruct(2, targetBlock, 0)
        fun endStateRequest(builder: FlatBufferBuilder) : Int {
            val o = builder.endTable()
            return o
        }
    }
}
