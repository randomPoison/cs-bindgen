using Xunit;

namespace TestRunner
{
    public class ValueTypes
    {
        [Fact]
        public void MahjongTile()
        {
            var tile = new SimpleTile(Suit.Bamboo, 1);
            Assert.Equal(Suit.Bamboo, tile.Suit);
            Assert.Equal(1, tile.Value);

            var result = IntegrationTests.RoundtripSimpleTile(tile);
            Assert.Equal(tile, result);
            Assert.Equal(tile.Suit, result.Suit);
            Assert.Equal(tile.Value, result.Value);
        }
    }
}
