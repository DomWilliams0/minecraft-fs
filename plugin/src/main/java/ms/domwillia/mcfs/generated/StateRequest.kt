// automatically generated by the FlatBuffers compiler, do not modify

package MCFS

import java.nio.*
import kotlin.math.sign
import com.google.flatbuffers.*

@Suppress("unused")
@ExperimentalUnsignedTypes
class StateRequest : Table() {

    fun __init(_i: Int, _bb: ByteBuffer)  {
        __reset(_i, _bb)
    }
    fun __assign(_i: Int, _bb: ByteBuffer) : StateRequest {
        __init(_i, _bb)
        return this
    }
    val entityList : Boolean
        get() {
            val o = __offset(4)
            return if(o != 0) 0.toByte() != bb.get(o + bb_pos) else false
        }
    companion object {
        fun validateVersion() = Constants.FLATBUFFERS_2_0_0()
        fun getRootAsStateRequest(_bb: ByteBuffer): StateRequest = getRootAsStateRequest(_bb, StateRequest())
        fun getRootAsStateRequest(_bb: ByteBuffer, obj: StateRequest): StateRequest {
            _bb.order(ByteOrder.LITTLE_ENDIAN)
            return (obj.__assign(_bb.getInt(_bb.position()) + _bb.position(), _bb))
        }
        fun createStateRequest(builder: FlatBufferBuilder, entityList: Boolean) : Int {
            builder.startTable(1)
            addEntityList(builder, entityList)
            return endStateRequest(builder)
        }
        fun startStateRequest(builder: FlatBufferBuilder) = builder.startTable(1)
        fun addEntityList(builder: FlatBufferBuilder, entityList: Boolean) = builder.addBoolean(0, entityList, false)
        fun endStateRequest(builder: FlatBufferBuilder) : Int {
            val o = builder.endTable()
            return o
        }
    }
}