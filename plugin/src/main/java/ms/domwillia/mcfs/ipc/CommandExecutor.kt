package ms.domwillia.mcfs.ipc

import MCFS.Command
import MCFS.CommandType
import MCFS.Error
import MCFS.Response
import com.google.flatbuffers.FlatBufferBuilder
import ms.domwillia.mcfs.MinecraftFsMod
import net.minecraft.client.MinecraftClient
import net.minecraft.client.network.ClientPlayerEntity
import java.nio.ByteBuffer

class NoGameException : Exception()

@ExperimentalUnsignedTypes
class CommandExecutor(private val responseBuilder: FlatBufferBuilder) {

    fun execute(command: Command): ByteBuffer {
        responseBuilder.clear();

        val resp = try {
            dewIt(command)
        } catch (_: NoGameException) {
            mkError(Error.NoGame)
        } catch (e: Exception) {
            MinecraftFsMod.LOGGER.catching(e)
            mkError(Error.Unknown)
        }

        responseBuilder.finish(resp)
        return responseBuilder.dataBuffer()
    }

    private fun dewIt(command: Command): Int {
        return when (command.cmd) {
            CommandType.PlayerHealth -> mkFloat(thePlayer.health)
            else -> {
                MinecraftFsMod.LOGGER.warn("Unknown command '$command'")
                mkError(Error.UnknownCommand)
            }
        }

    }

    private val thePlayer: ClientPlayerEntity
        get() = MinecraftClient.getInstance().player ?: throw NoGameException()


    private fun mkError(err: Int): Int {
        Response.startResponse(responseBuilder)
        Response.addError(responseBuilder, err)
        return Response.endResponse(responseBuilder)
    }

    private fun mkFloat(float: Float): Int {
        Response.startResponse(responseBuilder)
        Response.addFloat(responseBuilder, float)
        return Response.endResponse(responseBuilder)
    }
}