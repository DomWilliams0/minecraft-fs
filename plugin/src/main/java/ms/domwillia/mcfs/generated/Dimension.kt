// automatically generated by the FlatBuffers compiler, do not modify

package MCFS

@Suppress("unused")
class Dimension private constructor() {
    companion object {
        const val Overworld: UByte = 1u
        const val Nether: UByte = 2u
        const val End: UByte = 3u
        val names : Array<String> = arrayOf("Overworld", "Nether", "End")
        fun name(e: Int) : String = names[e - Overworld.toInt()]
    }
}
